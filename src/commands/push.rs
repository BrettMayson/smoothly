use std::path::PathBuf;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{Read, Write};

use md5::{Md5, Digest};
use pbo::PBO;
use sha1::{Sha1};

use crate::{SmoothlyError, Command, Repo, Addon, SwiftyFile, FilePart};

pub struct Push {}

impl Command for Push {
    fn register(&self) -> clap::App {
        clap::SubCommand::with_name("push")
            .about("Push the mods to an output directory")
            .arg(clap::Arg::with_name("dir")
                .help("Output directory")
                .required(true)
            ).arg(clap::Arg::with_name("mods")
                .help("Mods to push")
                .multiple(true)
                .takes_value(true)
            )
    }

    fn run(&self, args: &clap::ArgMatches, repo: String) -> Result<(), SmoothlyError> {
        let repo = Repo::new(repo)?;
        let dir = args.value_of("dir").unwrap();
        if !PathBuf::from(&dir).exists() {
            std::fs::create_dir_all(&dir)?;
        }

        let repofile = format!("{}{}repo.json", dir, std::path::MAIN_SEPARATOR);
        let mods: Vec<String> = if args.is_present("mods") {
            if !PathBuf::from(&repofile).exists() {
                println!("Unable to use selective push when no repo.json file exists");
                vec!()
            } else {
                args.values_of("mods").unwrap().map(|s| s.to_owned()).collect()
            }
        } else {
            vec!()
        };
        
        let mut outrepo = repo.clone();
        if !PathBuf::from(&repofile).exists() {
            File::create(&repofile)?;
        }

        let srcimage = format!("{}{}repo.png", repo.basePath, std::path::MAIN_SEPARATOR);
        if !PathBuf::from(&srcimage).exists() {
            println!("A repo.png is required. Add it to {}{} with dimensions of 300x160", repo.basePath, std::path::MAIN_SEPARATOR);
            std::process::exit(1);
        } else {
            let dst = format!("{}{}repo.png", dir, std::path::MAIN_SEPARATOR);
            std::fs::copy(srcimage, &dst)?;
            let mut image = File::open(&dst)?;
            let mut data = Vec::new();
            image.read_to_end(&mut data)?;
            let mut hasher = Sha1::new();
            hasher.input(&data);
            outrepo.imageChecksum = format!("{:X}", hasher.result());
        }

        let mut options = fs_extra::dir::CopyOptions::new();
        options.overwrite = true;
        options.skip_exist = false;

        println!("Transfering files");

        for entry in std::fs::read_dir(&repo.basePath)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_dir() { continue; }
            let name = path.file_name().unwrap().to_str().unwrap().to_owned();
            if !mods.is_empty() && !mods.contains(&name) { continue; }
            println!(" - {}", name);
            if repo.has_mod(&name) {
                let moddir = &format!("{}{}{}", dir, std::path::MAIN_SEPARATOR, name);
                std::fs::remove_dir_all(&moddir)?;
                std::fs::create_dir_all(&moddir)?;
                fs_extra::copy_items(&vec!(path), &format!("{}{}", dir, std::path::MAIN_SEPARATOR), &options).unwrap();
            }
        }

        println!("Generating SRFs");

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_dir() { continue; }
            let name = path.file_name().unwrap().to_str().unwrap().to_owned();
            if !mods.is_empty() && !mods.contains(&name) { continue; }
            println!(" - {}", name);
            let mut addon = Addon::new(name.clone());
            let moddir = &format!("{}{}{}", dir, std::path::MAIN_SEPARATOR, name);
            if repo.has_mod(&name) {
                for direntry in walkdir::WalkDir::new(&moddir).sort_by(|a,b| {
                    if a.path().is_dir() == b.path().is_dir() {
                        b.file_name().cmp(a.file_name())
                    } else {
                        b.path().is_dir().cmp(&a.path().is_dir())
                    }}) {
                    let entry = direntry.unwrap();
                    let path = entry.path();
                    if path.is_dir() { continue; }
                    let mut name = String::new();
                    let mut components = path.components();
                    components.next();components.next();
                    components.for_each(|e| {
                        match e {
                            std::path::Component::Normal(c) => {
                                name.push_str(&format!("{}\\", c.to_str().unwrap()));
                            },
                            _ => {}
                        }
                    });
                    name.pop();
                    let mut swiftyfile = SwiftyFile::new(name);
                    if path.extension().unwrap_or_else(|| OsStr::new("")) == OsStr::new("pbo") {
                        let pbo = PBO::read(&mut File::open(path).unwrap()).unwrap();
                        let mut headertotal = Vec::new();
                        headertotal.append(&mut vec!{0});
                        headertotal.append(&mut transform_u32_to_array_of_u8(0x5665_7273).to_vec().into_iter().rev().collect::<Vec<u8>>());
                        headertotal.append(&mut transform_u32_to_array_of_u8(0u32).to_vec().into_iter().rev().collect::<Vec<u8>>());
                        headertotal.append(&mut transform_u32_to_array_of_u8(0u32).to_vec().into_iter().rev().collect::<Vec<u8>>());
                        headertotal.append(&mut transform_u32_to_array_of_u8(0u32).to_vec().into_iter().rev().collect::<Vec<u8>>());
                        headertotal.append(&mut transform_u32_to_array_of_u8(0u32).to_vec().into_iter().rev().collect::<Vec<u8>>());

                        for header in pbo.extension_order {
                            headertotal.append(&mut header.chars().map(|c| c as u8).collect::<Vec<u8>>());
                            headertotal.append(&mut vec!{0});
                            headertotal.append(&mut pbo.extensions.get(&header).unwrap().chars().map(|c| c as u8).collect::<Vec<u8>>());
                            headertotal.append(&mut vec!{0});
                        }
                        headertotal.append(&mut vec!{0});
                        for header in pbo.headers {
                            // 4 bytes * 5 u32 per header
                            headertotal.append(&mut header.filename.chars().map(|c| c as u8).collect::<Vec<u8>>());
                            headertotal.append(&mut vec!{0});
                            headertotal.append(&mut transform_u32_to_array_of_u8(header.method).to_vec().into_iter().rev().collect::<Vec<u8>>());
                            headertotal.append(&mut transform_u32_to_array_of_u8(header.original).to_vec().into_iter().rev().collect::<Vec<u8>>());
                            headertotal.append(&mut transform_u32_to_array_of_u8(header.reserved).to_vec().into_iter().rev().collect::<Vec<u8>>());
                            headertotal.append(&mut transform_u32_to_array_of_u8(header.timestamp).to_vec().into_iter().rev().collect::<Vec<u8>>());
                            headertotal.append(&mut transform_u32_to_array_of_u8(header.size).to_vec().into_iter().rev().collect::<Vec<u8>>());
                        }
                        headertotal.append(&mut vec!{0});
                        headertotal.append(&mut transform_u32_to_array_of_u8(0u32).to_vec().into_iter().rev().collect::<Vec<u8>>());
                        headertotal.append(&mut transform_u32_to_array_of_u8(0u32).to_vec().into_iter().rev().collect::<Vec<u8>>());
                        headertotal.append(&mut transform_u32_to_array_of_u8(0u32).to_vec().into_iter().rev().collect::<Vec<u8>>());
                        headertotal.append(&mut transform_u32_to_array_of_u8(0u32).to_vec().into_iter().rev().collect::<Vec<u8>>());
                        headertotal.append(&mut transform_u32_to_array_of_u8(0u32).to_vec().into_iter().rev().collect::<Vec<u8>>());

                        let header_digest = {
                            let mut hasher = Md5::new();
                            hasher.input(&headertotal);
                            hasher.result()
                        };

                        swiftyfile.parts.push(FilePart {
                            name: "$$HEADER$$".to_owned(),
                            size: headertotal.len(),
                            hash: format!("{:X}", header_digest),
                            start: 0,
                        });

                        let mut start = headertotal.len();

                        for mut file in pbo.files {
                            let mut buffersize = 0;
                            let mut hasher = Md5::new();
                            loop {
                                let mut buffer = vec![0u8; 4194304];
                                let read = file.1.read(&mut buffer).unwrap();
                                hasher.input(&buffer[0..read]);
                                buffersize += read;
                                if read != 4194304 {
                                    break;
                                }
                            }
                            swiftyfile.parts.push(FilePart {
                                name: file.0.clone(),
                                size: buffersize,
                                hash: format!("{:X}",hasher.result()),
                                start,
                            });
                            start += buffersize;
                        }
                        let mut chk = pbo.checksum.unwrap();
                        chk.insert(0, 0);
                        swiftyfile.parts.push(FilePart {
                            name: "$$END$$".to_owned(),
                            size:  21,
                            hash: format!("{:X}", {
                                    let mut hasher = Md5::new();
                                    hasher.input(&chk);
                                    hasher.result()
                                }),
                            start,
                        });
                    } else {
                        if path.file_name().unwrap().to_str().unwrap() == "mod.srf" { continue; }
                        let mut f = File::open(path).unwrap();
                        let mut buffersize = 0;
                        let mut hasher = Md5::new();
                        loop {
                            let mut buffer = vec![0u8; 4194304];
                            let read = f.read(&mut buffer).unwrap();
                            hasher.input(&buffer[0..read]);
                            buffersize += read;
                            if read != 4194304 {
                                break;
                            }
                        }
                        swiftyfile.parts.push(FilePart {
                            name: format!("{}_{}", path.file_name().unwrap().to_str().unwrap().to_owned(), buffersize),
                            hash: format!("{:X}", hasher.result()),
                            size: buffersize as usize,
                            start: 0,
                        });
                    }
                    addon.files.push(swiftyfile);
                }

                let mut outfile = File::create(format!("{}{}mod.srf", moddir, std::path::MAIN_SEPARATOR))?;
                outfile.write_all(addon.line().as_bytes())?;
                for file in &mut addon.files {
                    outfile.write_all(file.line().as_bytes())?;
                }
                outrepo.set_hash(&name, addon.hash());
            }
        }

        println!("Generating Repofile");

        let j = serde_json::to_string_pretty(&outrepo).unwrap();
        let mut fout = File::create(repofile).unwrap();
        fout.write_all(j.as_bytes()).unwrap();

        Ok(())
    }
}

fn transform_u32_to_array_of_u8(x:u32) -> [u8;4] {
    let b1 : u8 = ((x >> 24) & 0xff) as u8;
    let b2 : u8 = ((x >> 16) & 0xff) as u8;
    let b3 : u8 = ((x >> 8) & 0xff) as u8;
    let b4 : u8 = (x & 0xff) as u8;
    return [b1, b2, b3, b4]
}
