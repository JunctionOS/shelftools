use jif::itree::interval::DataSource;
use jif::{Jif, PAGE_SIZE};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

type Sha256Hash = [u8; 32];

#[derive(Debug, Clone, Copy)]
pub(crate) struct DigestOptions {
    pub(crate) include_private: bool,
    pub(crate) include_shared: bool,
    pub(crate) ordering_only: bool,
}

#[derive(Default, Debug)]
pub(crate) struct JifDigest {
    private_pages: HashSet<Sha256Hash>,
    shared_pages: HashSet<(String, u64)>,
    zero_pages: usize,
}

fn sha256_page(page: &[u8]) -> Sha256Hash {
    let mut hasher = Sha256::new();
    hasher.update(page);
    hasher.finalize().into()
}

fn private_pages(jif: &Jif) -> HashSet<Sha256Hash> {
    let mut set = HashSet::new();
    jif.for_each_private_page(|page| {
        set.insert(sha256_page(page));
    });
    set
}

fn shared_pages(jif: &Jif) -> HashSet<(String, u64)> {
    jif.iter_shared_regions()
        .flat_map(|(string, start, end)| {
            (start..end)
                .step_by(PAGE_SIZE)
                .map(|addr| (string.to_string(), addr))
        })
        .collect()
}

fn ordering_digest(jif: &Jif, options: DigestOptions) -> JifDigest {
    let mut private = Vec::new();
    let mut shared = Vec::new();
    let mut zero_pages = 0;

    for page in jif.ord_chunks().iter().flat_map(|ord| ord.pages()) {
        match jif.source_at(page) {
            None => {
                eprintln!("{page:#x} is in the ordering section but is not mapped by the JIF");
            }
            Some(DataSource::Zero) => zero_pages += 1,
            Some(DataSource::Shared) if options.include_shared => {
                let pheader = jif
                    .mapping_pheader(page)
                    .expect("resolved shared pages must have a pheader");
                let offset_into_region = page - pheader.virtual_range().0;
                let filename = pheader
                    .pathname()
                    .expect("shared pages must have a filename")
                    .to_string();
                let ref_offset = pheader
                    .ref_offset()
                    .expect("shared pages must have a base file offset");
                shared.push((filename, ref_offset + offset_into_region));
            }
            Some(DataSource::Shared) => {}
            Some(DataSource::Private) if options.include_private => {
                let borrow = jif.resolve_data(page);
                let page_data = borrow.get().expect("resolved private pages must have data");

                assert_eq!(page_data.len(), PAGE_SIZE, "page is not page sized");
                private.push(sha256_page(page_data));
            }
            Some(DataSource::Private) => {}
        }
    }

    JifDigest {
        private_pages: private.into_iter().collect(),
        shared_pages: shared.into_iter().collect(),
        zero_pages,
    }
}

pub(crate) fn digest_jif(jif: &Jif, options: DigestOptions) -> JifDigest {
    if options.ordering_only {
        return ordering_digest(jif, options);
    }

    let mut digest = JifDigest::default();
    if options.include_private {
        digest.private_pages = private_pages(jif);
    }
    if options.include_shared {
        digest.shared_pages = shared_pages(jif);
    }
    digest.zero_pages = jif.zero_pages();
    digest
}

#[derive(Default, Debug)]
struct Stats {
    zero_pages: usize,
    private_pages: usize,
    truly_shared_pages: usize,
    unique_shared_pages: usize,
}

fn is_unique_shared_page(
    digests: &HashMap<PathBuf, JifDigest>,
    path: &Path,
    shared_page: &(String, u64),
) -> bool {
    digests
        .iter()
        .filter(|(other_path, _)| other_path.as_path() != path)
        .all(|(_, digest)| !digest.shared_pages.contains(shared_page))
}

fn percentage(parcel: usize, total: usize) -> f64 {
    if total == 0 {
        0.0
    } else {
        (parcel * 100) as f64 / total as f64
    }
}

pub(crate) fn print_intersections(digests: &HashMap<PathBuf, JifDigest>) {
    let mut stats = HashMap::new();
    for (path, digest) in digests {
        let mut stat = Stats {
            zero_pages: digest.zero_pages,
            private_pages: digest.private_pages.len(),
            ..Default::default()
        };

        for shared_page in &digest.shared_pages {
            if is_unique_shared_page(digests, path, shared_page) {
                stat.unique_shared_pages += 1;
            } else {
                stat.truly_shared_pages += 1;
            }
        }

        stats.insert(path, stat);
    }

    let max_width = stats
        .keys()
        .filter_map(|path| path.as_path().to_str().map(str::len))
        .chain(std::iter::once("filename".len()))
        .max()
        .unwrap_or("filename".len());

    println!(
        "{:^max_width$} | {:^8} | {:^15} | {:^15} | {:^15} | unique but shared |",
        "filename", "total", "zero", "private", "truly shared",
    );

    for (path, stat) in stats {
        let total = stat.zero_pages
            + stat.private_pages
            + stat.truly_shared_pages
            + stat.unique_shared_pages;
        println!(
            "{:max_width$} | {:8} | {:7} ({:4.1}%) | {:7} ({:4.1}%) | {:7} ({:4.1}%) | {:9} ({:4.1}%) |",
            path.display(),
            total,
            stat.zero_pages,
            percentage(stat.zero_pages, total),
            stat.private_pages,
            percentage(stat.private_pages, total),
            stat.truly_shared_pages,
            percentage(stat.truly_shared_pages, total),
            stat.unique_shared_pages,
            percentage(stat.unique_shared_pages, total)
        );
    }
}

pub(crate) fn write_upset_input(
    digests: HashMap<PathBuf, JifDigest>,
    writer: &mut dyn std::io::Write,
) -> anyhow::Result<()> {
    for (path, digest) in digests {
        write!(writer, "{}: ", path.display())?;

        for hash in &digest.private_pages {
            let hash = hash
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>();
            write!(writer, "private_{hash}, ")?;
        }

        for (pathname, offset) in digest.shared_pages {
            write!(writer, "shared_{pathname}_{offset:x}, ")?;
        }

        writeln!(writer)?;
    }

    Ok(())
}
