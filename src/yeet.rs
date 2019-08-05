use std::fs::File;
use std::io::Read;

use md5;
use pbo::PBO;

fn main() {
    let fpath = "C:\\Users\\Brett\\Documents\\swifty-test\\src\\@cba_a3\\optionals\\cba_jr_disable_long_scopes_on_short_mg_rail.pbo.cba_3.12.0.190708-15348392.bisign";
    if fpath.ends_with(".pbo") {
        println!("==PBO==");
        let pbo = PBO::read(&mut File::open(fpath).unwrap()).unwrap();
        let mut headertotal = Vec::new();
        headertotal.append(&mut vec!{0});
        headertotal.append(&mut transform_u32_to_array_of_u8(0x5665_7273).to_vec().into_iter().rev().collect::<Vec<u8>>());
        headertotal.append(&mut transform_u32_to_array_of_u8(0u32).to_vec().into_iter().rev().collect::<Vec<u8>>());
        headertotal.append(&mut transform_u32_to_array_of_u8(0u32).to_vec().into_iter().rev().collect::<Vec<u8>>());
        headertotal.append(&mut transform_u32_to_array_of_u8(0u32).to_vec().into_iter().rev().collect::<Vec<u8>>());
        headertotal.append(&mut transform_u32_to_array_of_u8(0u32).to_vec().into_iter().rev().collect::<Vec<u8>>());

        let mut hashes = Vec::new();

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
        hashes.append(&mut format!("{:X}", header_digest).chars().map(|c| c as u8).collect::<Vec<u8>>());
        println!("$$HEADER$$ {} {:X}", headertotal.len(), header_digest);
        for mut file in pbo.files {
            println!("{}", file.0);
            let mut buffer = Vec::new();
            file.1.read_to_end(&mut buffer).unwrap();
            let digest = md5::compute(&buffer);
            hashes.append(&mut format!("{:X}", digest).chars().map(|c| c as u8).collect::<Vec<u8>>());
            println!("{:X}", digest);
        }
        let mut chk = pbo.checksum.unwrap();
        chk.insert(0, 0);
        let digest = md5::compute(&chk);
        println!("$$END$$ {:X}", digest);
        hashes.append(&mut format!("{:X}", digest).chars().map(|c| c as u8).collect::<Vec<u8>>());
        let digest = md5::compute(&hashes);
        println!("{:X}", digest);
    } else {
        let mut f = File::open(fpath).unwrap();
        let mut buffer = Vec::new();
        f.read_to_end(&mut buffer).unwrap();
        let digest = md5::compute(&buffer);
        println!("{:X}", digest);
        let digest = md5::compute(format!("{:X}",&digest).chars().map(|c| c as u8).collect::<Vec<u8>>());
        println!("{:X}", digest);
    }
}

fn transform_u32_to_array_of_u8(x:u32) -> [u8;4] {
    let b1 : u8 = ((x >> 24) & 0xff) as u8;
    let b2 : u8 = ((x >> 16) & 0xff) as u8;
    let b3 : u8 = ((x >> 8) & 0xff) as u8;
    let b4 : u8 = (x & 0xff) as u8;
    return [b1, b2, b3, b4]
}
