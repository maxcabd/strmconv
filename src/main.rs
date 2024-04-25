mod convert;

use std::path::Path;
use xfbin::nucc_chunk::NuccChunkType;
use xfbin::nucc::{NuccCamera, NuccLightDirc, NuccLightPoint, NuccLayerSet, NuccAmbient, NuccMorphModel, NuccStruct, NuccStructInfo, NuccStructReference};
use xfbin::{read_xfbin, write_xfbin};
use xfbin::{Xfbin, xfbin::XfbinPage};
use convert::convert_anmstrm;


const CHUNK_TYPES_TO_ADD: [NuccChunkType; 6] = [
    NuccChunkType::NuccChunkCamera,
    NuccChunkType::NuccChunkLightDirc,
    NuccChunkType::NuccChunkLightPoint,
    NuccChunkType::NuccChunkLayerSet,
    NuccChunkType::NuccChunkAmbient,
    NuccChunkType::NuccChunkMorphModel
];


fn main() {
    let time = std::time::Instant::now();
    
    let args: Vec<String> = std::env::args().collect();
    let filepath = Path::new(&args[1]);

    let xfbin = read_xfbin(&filepath).unwrap();
    println!("Converting file: {:?}", filepath.file_name().unwrap().to_str().unwrap());

    let mut structs_to_add: Vec<Box<dyn NuccStruct>> = vec![];

    for chunk_type in &CHUNK_TYPES_TO_ADD {
        let nucc_structs = find_nucc_structs(&xfbin, chunk_type.clone());

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

                    NuccChunkType::NuccChunkMorphModel => {
                        if let Some(morphmodel) = nucc_struct.downcast_ref::<NuccMorphModel>() {
                            structs_to_add.push(Box::new(morphmodel.clone()));
                        }
                    }
                    _ => {}
                }
            }
    }
    
    let anm_chunk_name = filepath.file_stem().unwrap().to_str().unwrap().split('.').next().unwrap();

    let (anmstrm_info, anm_struct_references) = get_page_info(&xfbin, anm_chunk_name);

    let anm_struct_infos = xfbin.pages[0].struct_infos.clone();

    let dmg_anm_info = NuccStructInfo {
        chunk_name: anm_chunk_name.to_string() + "_dmg",
        chunk_type: NuccChunkType::NuccChunkAnm.to_string(),
        filepath: anmstrm_info.filepath.clone().replace(anm_chunk_name, &(anm_chunk_name.to_string() + "_dmg")),
    };

    let mut new_xfbin = Xfbin::default();

    let converted_structs = convert_anmstrm(&xfbin, &anmstrm_info, &dmg_anm_info)
    .unwrap()
    .iter()
    .cloned()
    .collect::<Vec<_>>();

    let mut anm_page = XfbinPage::default();
    anm_page.struct_infos = anm_struct_infos.clone();
    anm_page.struct_infos.push(anmstrm_info.clone());
    anm_page.struct_references = anm_struct_references.clone();
    anm_page.structs.extend(structs_to_add);
    anm_page.structs.push(Box::new(converted_structs[0].clone()) as Box<dyn NuccStruct>);

    let mut dmg_anm_page = XfbinPage::default();
    dmg_anm_page.struct_infos = anm_struct_infos;
    dmg_anm_page.struct_infos.push(dmg_anm_info.clone());
    dmg_anm_page.struct_references = anm_struct_references.clone();

    dmg_anm_page.structs = vec![Box::new(converted_structs[1].clone()) as Box<dyn NuccStruct>];
   
    new_xfbin.pages.push(anm_page);
    new_xfbin.pages.push(dmg_anm_page);
    

    let converted_filename = anm_chunk_name.to_string() + ".anm.xfbin";
    write_xfbin(new_xfbin, &Path::new(converted_filename.as_str())).unwrap();

 
    println!("Finished converting strm to anm in file '{}' in {:?}s", time.elapsed().as_secs_f64(), converted_filename);


}

fn find_nucc_structs(xfbin: &Xfbin, chunk_type: NuccChunkType) -> Vec<&Box<dyn NuccStruct>> {
    xfbin.pages.iter().flat_map(|page| {
        page.structs.iter().filter_map(|nucc_struct| {
            if nucc_struct.chunk_type() == chunk_type {
                Some(nucc_struct)
            } else {
                None
            }
        })
    }).collect()
}

fn get_page_info<'a>(xfbin: &'a Xfbin, chunk_name: &'a str) -> (NuccStructInfo, Vec<NuccStructReference>) {
    let anm_struct_references = xfbin.pages.iter().flat_map(|page| {
        page.struct_references.iter().filter_map(|nucc_struct_ref| {
            if let NuccChunkType::NuccChunkAnmStrm = NuccChunkType::NuccChunkAnmStrm {
                Some(nucc_struct_ref.clone())
            } else {
                None
            }
        })
    }).collect::<Vec<_>>();

    let mut anmstrm_info: NuccStructInfo = xfbin.pages.iter().flat_map(|page| {
        page.struct_infos.iter().filter_map(|nucc_struct_info| {
            if nucc_struct_info.chunk_name == chunk_name && nucc_struct_info.chunk_type == NuccChunkType::NuccChunkAnmStrm.to_string() {
                Some(nucc_struct_info.clone())
            } else {
                None
            }
        })
    }).next().unwrap();

    anmstrm_info.chunk_type = NuccChunkType::NuccChunkAnm.to_string();

    (anmstrm_info, anm_struct_references)
}