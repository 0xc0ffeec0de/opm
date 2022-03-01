use anyhow::Result;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle, HumanBytes};
use xz2::read::XzDecoder;
use flate2::read::GzDecoder;
use std::{
    io::prelude::*,
    path::Path,
    str
};
use futures::{future, StreamExt};
use super::sources::DebianSource;
use crate::repos::{
    os_fingerprint::Archs::*,
    config::Config,
};

fn unpack(filename: &str, data: &[u8], bytes: &mut Vec<u8>) {
    if filename.ends_with(".gz") {
        let mut tar = GzDecoder::new(data);
        tar.read_to_end(bytes).unwrap_or_default();
    } else if filename.ends_with(".xz") {
        let mut tar = XzDecoder::new(data);
        tar.read_to_end(bytes).unwrap_or_default();
    }
}

pub fn clear(config: &Config) -> Result<()> {
    fs_extra::dir::create(&config.cache, true)?;
    fs_extra::dir::create(&config.rls, true)?;
    fs_extra::dir::create(&config.tmp, true)?;
    Ok(())
}

const fn os_arch(config: &Config) -> &'static str {
    match &config.os_info.arch {
        Amd64 => "binary-amd64",
        I386 => "binary-i386",
        _ => panic!("Unknown architecture"),
    }
}

pub async fn update(config: &mut Config, repos: &[DebianSource]) -> Result<()> {
    let (mut cache, mut rls) = (vec![], vec![]);
    let spinner_style = ProgressStyle::default_spinner()
        .template("{spinner} {prefix}");

    let mp = MultiProgress::new();
    for (i, source) in repos.iter().enumerate() {
        for perm in source.components.iter() {
            let cache_bar = mp.add(ProgressBar::new(0));
            cache_bar.set_style(spinner_style.clone());

            let rls_bar = mp.add(ProgressBar::new(0));
            rls_bar.set_style(spinner_style.clone());
            
            cache.push(update_cache(config, &source.url, &source.distribution, perm, cache_bar, i));
            rls.push(update_releases(config, &source.url, &source.distribution, perm, rls_bar, i));
        }
    }
    let handle = tokio::task::spawn_blocking(move || mp.join().unwrap());

    future::join_all(rls).await;
    future::join_all(cache).await;
    
    handle.await?;

    Ok(())
}

async fn update_cache(config: &Config, url: &str, dist: &str, perm: &str, pb: ProgressBar, counter: usize) -> Result<()> {
    let pgp = format!("{}dists/{}/Release.gpg", url, dist);
    if reqwest::get(&pgp).await.is_err() {
        panic!("Could not verify the repository due missing PGP Signarure (URL: {})", url);
    };
    // Binary packages ONLY for now
    let pkgcache = format!("{}dists/{}/{}/{}/Packages.xz", url, dist, perm, os_arch(config));
    let response = match reqwest::get(&pkgcache).await {
        Ok(r) => Some(r),
        Err(_) => {
            let pkgcache = format!("{}dists/{}/{}/{}/Packages.gz", url, dist, perm, os_arch(config));
            match reqwest::get(&pkgcache).await {
                Ok(r) => Some(r),
                Err(e) => {
                    eprintln!("Could not get the package at {} due {}", pkgcache, e);
                    None
                }
            }
        }
    };

    let url = url.replace("http://", "");
    let url = url.replace("/", "_");    
    
    if let Some(response) = response {
        let size = response.content_length().unwrap_or_default();
        pb.set_length(size);
        pb.set_prefix(format!("{}: {} [{}]", counter+1, pkgcache, HumanBytes(size)));

        let (mut stream, mut downloaded) = (response.bytes_stream(), 0u64);
        
        let mut content = Vec::with_capacity(size as usize);
        let pkg = Path::new(&config.cache).join(format!("{}dists_{}_{}_{}_Packages", url, dist, perm, os_arch(config)));

        while let Some(item) = stream.next().await {
            let chunk = item?;
            Write::write(&mut content, &chunk)?;
            let progress = std::cmp::min(downloaded + chunk.len() as u64, size);
            downloaded = progress;
            pb.set_position(progress);
        }
        
        let mut bytes = Vec::new();
        unpack(&pkgcache, content.as_ref(), &mut bytes);
        let mut bytes: &[u8] = bytes.as_ref();
        if !bytes.is_empty() {
            let mut pkg = tokio::fs::File::create(pkg).await.unwrap();
            tokio::io::copy(&mut bytes, &mut pkg).await.unwrap();
        }

        pb.finish_and_clear()
    }
    
    Ok(())
}

async fn update_releases(config: &Config, url: &str, dist: &str, perm: &str, pb: ProgressBar, counter: usize) -> Result<()> {

    let release_file = format!("{}dists/{}/InRelease", url, dist);

    let url = str::replace(url, "http://", "");
    let url = str::replace(&url, "/", "_");
    
    let response = reqwest::get(&release_file).await?;
    let size = response.content_length().unwrap_or_default();
    pb.set_length(size);
    pb.set_prefix(format!("{}: {} [{}]", counter+1, release_file, HumanBytes(size)));
    
    let (mut stream, mut downloaded) = (response.bytes_stream(), 0u64);

    let mut content = Vec::with_capacity(size as usize);
    let rls = Path::new(&config.rls).join(format!("{}dists_{}_{}_{}_InRelease", url, dist, perm, os_arch(config)));

    while let Some(item) = stream.next().await {
        let chunk = item?;
        Write::write(&mut content, &chunk)?;
        let progress = std::cmp::min(downloaded + chunk.len() as u64, size);
        downloaded = progress;
        pb.set_position(progress);
    }
    
    
    let mut dest = tokio::fs::File::create(rls).await.unwrap();
    tokio::io::copy(&mut content.as_ref(), &mut dest).await.unwrap();

    pb.finish_and_clear();
    Ok(())
}
