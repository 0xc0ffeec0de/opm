use anyhow::Result;
use xz2::read::XzDecoder;

use reqwest;

use std::{io::{ErrorKind, prelude::*}, path::Path};
use std::fs;
use std::str;
use futures::future;

use super::sources::DebianSource;
use crate::repos::config::Config;

fn clear(config: &Config) -> Result<()> {
    match fs::remove_dir_all(&config.cache){
        Ok(_) => (),
        Err(e) => match e.kind() {
            ErrorKind::AlreadyExists => (),
            ErrorKind::NotFound => (),
            _ => panic!("fuck {}", e)
        }
    }
    match fs::remove_dir_all(&config.rls){
        Ok(_) => (),
        Err(e) => match e.kind() {
            ErrorKind::AlreadyExists => (),
            ErrorKind::NotFound => (),
            _ => panic!("fuck {}", e)
        }
    }
    match fs::remove_dir_all(&config.tmp){
        Ok(_) => (),
        Err(e) => match e.kind() {
            ErrorKind::AlreadyExists => (),
            ErrorKind::NotFound => (),
            _ => panic!("fuck {}", e)
        }
    }

    fs::create_dir(&config.cache)?;
    fs::create_dir(&config.rls)?;
    fs::create_dir(&config.tmp)?;

    Ok(())
}

pub async fn update(config: &mut Config, repos: &[DebianSource]) -> Result<()> {
    clear(config)?;

    update_releases(config, repos).await?;
    update_cache(config, repos).await?;

    Ok(())
}

async fn update_cache(config: &Config, repos: &[DebianSource]) -> Result<()> {
    let mut tasks = vec![];
    for (i, source) in repos.iter().enumerate() {
        println!("Get {}: {} {} {:?}", i+1, source.url, source.distribution, source.components);
        for perm in source.components.iter() {
            let pkgcache = format!("{}dists/{}/{}/binary-amd64/Packages.xz", source.url, source.distribution, perm); // Binary packages ONLY for now

            let url = str::replace(&source.url, "http://", "");
            let url = str::replace(&url, "/", "_");

            let pkg = Path::new(&config.cache).join(format!("{}{}_{}_binary-amd64_Packages", url, source.distribution, perm));
            
            tasks.push(tokio::spawn(async move {
                let response = reqwest::get(pkgcache).await.unwrap();

                let content = response.bytes().await.unwrap();
                let content: &[u8] = content.as_ref();
                
                let mut data = XzDecoder::new(content);
                let mut bytes = Vec::new();

                data.read_to_end(&mut bytes).unwrap_or_default();
                
                let mut bytes: &[u8] = bytes.as_ref();
                
                let mut pkg = tokio::fs::File::create(pkg).await.unwrap();
                tokio::io::copy(&mut bytes, &mut pkg).await.unwrap();
            }));
        };
    }

    future::join_all(tasks).await;
    Ok(())
}

async fn update_releases(config: &Config, repos: &[DebianSource]) -> Result<()> {
    let mut tasks = vec![];
    for (i, source) in repos.iter().enumerate() {
        println!("RLS {}: {} {} {:?}", i+1, source.url, source.distribution, source.components);
        for perm in source.components.iter() {
            let release_file = format!("{}dists/{}/InRelease", source.url, source.distribution);
            
            let url = str::replace(&source.url, "http://", "");
            let url = str::replace(&url, "/", "_");
            
            let rls = Path::new(&config.rls).join(format!("{}{}_{}_binary-amd64_InRelease", url, source.distribution, perm));
            tasks.push(tokio::spawn(async move {
                let response = reqwest::get(release_file).await.unwrap();
                let mut dest = tokio::fs::File::create(rls).await.unwrap();
                let content =  response.text().await.unwrap();
                tokio::io::copy(&mut content.as_bytes(), &mut dest).await.unwrap();
            }));
        }
    }

    future::join_all(tasks).await;
    Ok(())
}