mod converter;

use std::fs;
use std::path::Path;

use xfbin::nucc::{
    NuccAmbient, NuccCamera, NuccLayerSet, NuccLightDirc, NuccLightPoint, NuccMorphModel,
    NuccStruct, NuccStructInfo, NuccStructReference,
};
use xfbin::nucc_chunk::NuccChunkType;
use xfbin::{read_xfbin, write_xfbin};
use xfbin::{xfbin::XfbinPage, Xfbin};

use converter::convert_anmstrm;
use xml2fcv::{create_fcv_xfbin, get_frame_settings};

const CHUNK_TYPES_TO_ADD: [NuccChunkType; 6] = [
    NuccChunkType::NuccChunkCamera,
    NuccChunkType::NuccChunkLightDirc,
    NuccChunkType::NuccChunkLightPoint,
    NuccChunkType::NuccChunkMorphModel,
    NuccChunkType::NuccChunkLayerSet,
    NuccChunkType::NuccChunkAmbient,
];

fn main() {
    let time = std::time::Instant::now();

    let args: Vec<String> = std::env::args().collect();

    if args.len() != 3 {
        panic!(
            "Invalid number of arguments! Expected 3, got {}",
            args.len() - 1
        );
    }

    let filepath = Path::new(args.iter().find(|arg| arg.ends_with(".xfbin")).unwrap());

    let xfbin = read_xfbin(&filepath).unwrap();
    println!(
        "Converting file: {:?}",
        filepath.file_name().unwrap().to_str().unwrap()
    );

    let mut structs_to_add: Vec<Box<dyn NuccStruct>> = vec![];

    for chunk_type in &CHUNK_TYPES_TO_ADD {
        let nucc_structs = xfbin.find_nucc_structs(chunk_type.clone());

        for nucc_struct in nucc_structs {
            match chunk_type {
                NuccChunkType::NuccChunkCamera => {
                    if let Some(camera) = nucc_struct.downcast_ref::<NuccCamera>() {
                        structs_to_add.push(Box::new(camera.clone()));
                    }
                }
                NuccChunkType::NuccChunkLightDirc => {
                    if let Some(lightdirc) = nucc_struct.downcast_ref::<NuccLightDirc>() {
                        structs_to_add.push(Box::new(lightdirc.clone()));
                    }
                }
                NuccChunkType::NuccChunkLightPoint => {
                    if let Some(lightpoint) = nucc_struct.downcast_ref::<NuccLightPoint>() {
                        structs_to_add.push(Box::new(lightpoint.clone()));
                    }
                }
                NuccChunkType::NuccChunkMorphModel => {
                    if let Some(morphmodel) = nucc_struct.downcast_ref::<NuccMorphModel>() {
                        structs_to_add.push(Box::new(morphmodel.clone()));
                    }
                }
                NuccChunkType::NuccChunkLayerSet => {
                    if let Some(layerset) = nucc_struct.downcast_ref::<NuccLayerSet>() {
                        structs_to_add.push(Box::new(layerset.clone()));
                    }
                }
                NuccChunkType::NuccChunkAmbient => {
                    if let Some(ambient) = nucc_struct.downcast_ref::<NuccAmbient>() {
                        structs_to_add.push(Box::new(ambient.clone()));
                    }
                }

                _ => {}
            }
        }
    }

    let anm_chunk_name = filepath
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap()
        .split('.')
        .next()
        .unwrap();

    let (anmstrm_info, anm_struct_references) = get_page_info(&xfbin, anm_chunk_name);

    let anm_struct_infos = xfbin.pages[0].struct_infos.clone();

    let mut dmg_struct_infos = anm_struct_infos.clone();
    let mut dmg_references = anm_struct_references.clone();

    let dmg_anm_info = NuccStructInfo {
        chunk_name: anm_chunk_name.to_string() + "_dmg",
        chunk_type: NuccChunkType::NuccChunkAnm.to_string(),
        filepath: anmstrm_info
            .filepath
            .clone()
            .replace(anm_chunk_name, &(anm_chunk_name.to_string() + "_dmg")),
    };

    let mut new_xfbin = Xfbin::default();

    let converted_structs = convert_anmstrm(
        &xfbin,
        &anmstrm_info,
        &dmg_anm_info,
        &mut dmg_references,
        &mut dmg_struct_infos,
    )
    .unwrap()
    .iter()
    .cloned()
    .collect::<Vec<_>>();

    let mut anm_page = XfbinPage::default();
    anm_page.struct_infos = anm_struct_infos.clone();
    anm_page.struct_infos.push(anmstrm_info.clone());
    anm_page.struct_references = anm_struct_references.clone();
    anm_page.structs.extend(structs_to_add);
    anm_page
        .structs
        .push(Box::new(converted_structs[0].clone()) as Box<dyn NuccStruct>);

    let mut dmg_anm_page = XfbinPage::default();
    dmg_anm_page.struct_infos = dmg_struct_infos;
    dmg_anm_page.struct_infos.push(dmg_anm_info.clone());
    dmg_anm_page.struct_references = dmg_references;
    dmg_anm_page.structs = vec![Box::new(converted_structs[1].clone()) as Box<dyn NuccStruct>];

    new_xfbin.pages.push(anm_page);
    new_xfbin.pages.push(dmg_anm_page);

    let xml_path = args.iter().find(|arg| arg.ends_with(".xml")).unwrap();
    let frame_settings = get_frame_settings(fs::read_to_string(xml_path).unwrap().as_str());
    println!(
        "Parsing fcurve settings from '{}'",
        Path::new(xml_path).file_name().unwrap().to_str().unwrap()
    );

    let mut fcv_xfbin = Xfbin::default();
    create_fcv_xfbin(&mut fcv_xfbin, &frame_settings, anm_chunk_name);

    for page in fcv_xfbin.pages {
        new_xfbin.pages.push(page);
    }

    let converted_filename = anm_chunk_name.to_string() + ".anm.xfbin";
    write_xfbin(new_xfbin, &Path::new(converted_filename.as_str())).unwrap();

    println!(
        "Finished converting strm to anm file '{}' in {:?}s \n",
        converted_filename,
        time.elapsed().as_secs_f64()
    );
}

fn get_page_info<'a>(
    xfbin: &'a Xfbin,
    chunk_name: &'a str,
) -> (NuccStructInfo, Vec<NuccStructReference>) {
    let anm_struct_references = xfbin
        .pages
        .iter()
        .flat_map(|page| {
            page.struct_references.iter().filter_map(|nucc_struct_ref| {
                if let NuccChunkType::NuccChunkAnmStrm = NuccChunkType::NuccChunkAnmStrm {
                    Some(nucc_struct_ref.clone())
                } else {
                    None
                }
            })
        })
        .collect::<Vec<_>>();

    let mut anmstrm_info: NuccStructInfo = xfbin
        .pages
        .iter()
        .flat_map(|page| {
            page.struct_infos.iter().filter_map(|nucc_struct_info| {
                if nucc_struct_info.chunk_name == chunk_name
                    && nucc_struct_info.chunk_type == NuccChunkType::NuccChunkAnmStrm.to_string()
                {
                    Some(nucc_struct_info.clone())
                } else {
                    None
                }
            })
        })
        .next()
        .unwrap();

    anmstrm_info.chunk_type = NuccChunkType::NuccChunkAnm.to_string();

    (anmstrm_info, anm_struct_references)
}
