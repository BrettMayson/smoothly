use std::path::PathBuf;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{Read, Write};

use rayon::prelude::*;
use pbo::PBO;

use crate::{SmoothlyError, Command, Repo, Addon, SwiftyFile, FilePart};

pub struct Push {}

impl Command for Push {
    fn register(&self) -> (&str, clap::App) {
        ("push",
            clap::SubCommand::with_name("push")
                .about("Push the mods to an output directory")
                .arg(clap::Arg::with_name("dir")
                    .help("Output directory")
                    .required(true)
                )
        )
    }

    fn run(&self, args: &clap::ArgMatches, repo: String) -> Result<(), SmoothlyError> {
        let repo = Repo::new(repo)?;
        let dir = args.value_of("dir").unwrap();
        if !PathBuf::from(&dir).exists() {
            std::fs::create_dir_all(&dir)?;
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
            if repo.has_mod(&name) {
                fs_extra::copy_items(&vec!(path), &format!("{}{}", dir, std::path::MAIN_SEPARATOR), &options).unwrap();
            }
        }

        println!("Generating SRFs");

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_dir() { continue; }
            let name = path.file_name().unwrap().to_str().unwrap().to_owned();
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

                        let header_digest = md5::compute(&headertotal);

                        swiftyfile.parts.push(FilePart {
                            name: "$$HEADER$$".to_owned(),
                            size: headertotal.len(),
                            hash: format!("{:X}", header_digest),
                            start: 0,
                        });

                        let mut start = headertotal.len();

                        for mut file in pbo.files {
                            let mut buffer = Vec::new();
                            file.1.read_to_end(&mut buffer).unwrap();
                            swiftyfile.parts.push(FilePart {
                                name: file.0.clone(),
                                size: buffer.len(),
                                hash: format!("{:X}", md5::compute(&buffer)),
                                start,
                            });
                            start += buffer.len();
                        }
                        let mut chk = pbo.checksum.unwrap();
                        chk.insert(0, 0);;
                        swiftyfile.parts.push(FilePart {
                            name: "$$END$$".to_owned(),
                            size:  21,
                            hash: format!("{:X}", md5::compute(&chk)),
                            start,
                        });
                    } else {
                        if path.file_name().unwrap().to_str().unwrap() == "mod.srf" { continue; }
                        let mut f = File::open(path).unwrap();
                        let mut buffer = Vec::new();
                        f.read_to_end(&mut buffer).unwrap();
                        swiftyfile.parts.push(FilePart {
                            name: format!("{}_{}", path.file_name().unwrap().to_str().unwrap().to_owned(), buffer.len()),
                            hash: format!("{:X}", md5::compute(&buffer)),
                            size: buffer.len(),
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
            }
        }
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
