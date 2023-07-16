use std::{env, string, path};
use std::{fs, vec};
use std::path::Path;
use osu_db::listing::{Listing, Beatmap};
use id3::{Tag, TagLike, Version};
use std::ffi::OsStr;
use id3::frame::{Picture, PictureType};
use std::fs::File;
use std::io::Read;



const GAME_PATH: &str = "C:/Users/coole_yzbt6o8/AppData/Local/osu!";
const DEST_FOLDER: &str = "E:/files/music";


#[derive(Debug)]
enum Errors {
    CopyError,
    ExtensionError,
    DatabaseError,
    TagError,
}

struct Configuration <'a> {
    game_path: Option<&'a str>,
    dest_folder: Option<&'a str>,
    include_picture: bool,
    unicode_titles: bool,
}


fn main() {
    //handle_flags();

    //ask_path();
    println!("parsing trough database..");
    // Load the listing to memory
    let mut listing = Listing::from_file(GAME_PATH.to_owned()+"/osu!.db").unwrap();
    println!("parsing terminated without error");

    // Contains all the songs already copied
    let mut songs_done:Vec<String> = Vec::new();

    // create default string in case beatmap.foldername or beatmap.audio are set to None
    let mut fallback_string = String::from("");
    let reference_fallback_string = &mut fallback_string;
    
    // create index to keep track of progression
    let mut i: i64 = 0;
    
    let mut failed_songs: i64 = 0;
    let size = listing.beatmaps.len();



    // loop through all maps in database
    for beatmap in listing.beatmaps.iter_mut() {
       
        // Checking if the song was already copied
        let song: String = beatmap.folder_name.as_mut().unwrap_or_else(|| reference_fallback_string).as_str().to_owned() + beatmap.audio.as_mut().unwrap_or_else(|| reference_fallback_string).as_str();
        if !songs_done.contains(&song) {
            print!("({}/{}) processing beatmap {} - {} ", i, size,  beatmap.title_ascii.as_mut().unwrap(), beatmap.difficulty_name.as_mut().unwrap());
        
            match mainloop(beatmap) {
                Err(err) => {println!("encountered error {:?}", err); failed_songs += 1},
                Ok(()) => println!("{}", "successfull")
            };
            songs_done.push(song);
        }
        i += 1;

    }
    println!("operation done, failed {}", failed_songs);
}

/*
fn handle_flags() -> Configuration <'static>{
    let mut conf: Configuration = Configuration { game_path: None,
        dest_folder: None,
        include_picture: false,
        unicode_titles: false };
  
    let argstring: Vec<String> = env::args().skip(1).collect();
    let args: Vec<&str> = argstring.iter().map(|s| &**s).collect();
    println!("{:?}", args);
    let mut i = 0;

    loop {
        if i > args.len() {
            break;
        }
        match args[i] {
            "-i" | "-input" => {conf.game_path = Some(&args[i+1]); i += 2},
            "-o" | "-output" => {conf.dest_folder = Some(&args[i+1]); i += 2},
            "-p" | "-picture" => {conf.include_picture = true; i+=1},
            "-u" | "unicode" => {conf.unicode_titles = true; i+=1},
            _ => panic!("unknown flag")

        }
    }
    return conf

} */
fn mainloop(beatmap: &mut Beatmap) -> Result<(), Errors> {
    
    if beatmap.audio == None || beatmap.folder_name == None {
        
        return Err(Errors::DatabaseError);
    }
    let audio = beatmap.audio.as_mut().unwrap().as_str();
        
    
    if  get_extension_from_filename(audio) == None {
        return Err(Errors::ExtensionError);
    }
    let extension = get_extension_from_filename(audio).unwrap();

    // remove forbidden chars of files and folders in os
    let dest_audio_file_name: String = beatmap.title_ascii.as_mut().unwrap().as_str().replace(&['<', '>', '/', '\\', '*', '?', '\"', ':', '|'][..], "") + "." +extension;

    // set the source path and the destination path to prepare copy
    let src_path: &str = &(GAME_PATH.to_owned() + "/Songs/" + beatmap.folder_name.as_mut().unwrap().as_str() + "/" + audio);
    let dst_path = DEST_FOLDER.to_owned()+"/"+dest_audio_file_name.as_str();


    copy_file(src_path.clone(), dst_path.clone().as_str())?;

    // TODO: add more audio formats to edit metadata
    match extension {
    
        "mp3" => set_metadata_id3(&dst_path, beatmap)?,
        _ => {println!("unsupported extension"); return Err(Errors::TagError)},
        
    }

    return Ok(())

     

}



/**
 * Copy a file without overwritting 
 */
fn copy_file(src_path: &str, dest_path: &str) -> Result<String, Errors> {
    if !Path::new(src_path).exists() {
        println!("path not found {}", src_path);
        return Err(Errors::CopyError);
    }


    if !Path::new(dest_path).exists() {
        match fs::copy(src_path, dest_path) {
            Ok(_) => return Ok(dest_path.to_string()),
            Err(error) =>  {println!("error while try to copy : {:?}", error);
                                   return Err(Errors::CopyError)},

        }
    } else { // if the file already exists, compare files and copy if they are not the same
        // this can prevent for instance full song overwritten by TV size songs
        // this method will keep both
        if  fs::metadata(src_path).unwrap().len() != fs::metadata(dest_path).unwrap().len() {
            // recursively try to copy song if the file already exists
            // each time it appends the extension of the file 
            copy_file(src_path, &(src_path.to_owned() +get_extension_from_filename(src_path).unwrap_or_else(|| " ")))?;
        }
    }
    Ok(dest_path.to_string())
}

fn set_metadata_id3(audio_path:&str, beatmap: &mut Beatmap) -> Result<(), Errors>{
    let mut tag = Tag::new();

    tag.set_title(beatmap.title_ascii.as_mut().unwrap().as_str());
    tag.set_artist(beatmap.artist_ascii.as_mut().unwrap().as_str());
    tag.set_genre("tags:".to_owned()+beatmap.tags.as_mut().unwrap().as_str());
    

    // path of the background
    let bg_image_path = GAME_PATH.to_owned() +"/Data/bt/"+ beatmap.beatmapset_id.to_string().as_str()+ ".jpg";
    
    if Path::new(&bg_image_path).exists() {
        
        // fetching data from the background
        let mut f = File::open(bg_image_path).unwrap();
        let mut data = vec![];
        f.read_to_end(&mut data).unwrap();
        
        tag.add_frame(Picture {
            mime_type: "image/jpg".to_string(),
            picture_type: PictureType::Other,
            description: "beatmap background".to_string(),
            data: data,
        });
        
    }
    
    
    
    match tag.write_to_path(audio_path, Version::Id3v24) {
        Err(_result) => return Err(Errors::TagError),
        Ok(()) => return  Ok(())

    };
}



/**
  Get the extension of a file given its path
 */
fn get_extension_from_filename(filename: &str) -> Option<&str> {
    Path::new(filename)
        .extension()
        .and_then(OsStr::to_str)
}


