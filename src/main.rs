extern crate csv;
#[macro_use]
extern crate error_chain;
extern crate serde;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate slime;
extern crate toml;

use serde_json::Value as SerdeJson;
use std::fs::File;
use std::path::Path;
use std::io::prelude::*;
use std::collections::HashMap;

error_chain! {
    foreign_links {
        CsvReader(csv::Error);
        JsonError(serde_json::Error);
        Io(std::io::Error);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Idea {
    title: String,
    description: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct IdeaWrapper {
    idea: Idea,
    name: String,
    votes: u32,
}



fn main() {
    let votes = get_all_votes().expect("failed to parse votes");
    let ideas = get_ideas().expect("failed to get ideas");
    for (k, v) in &votes {

        println!("votes: {} : {}", &k, &v);
    }

    generate_votes_page(votes, ideas).expect("failed to generate page");
}

fn get_all_votes() -> Result<HashMap<String, u32>> {
    let mut hm: HashMap<String, u32> = HashMap::new();
    let paths = get_all_vote_file_paths().expect("failed to get all votes file paths");
    for p in paths {
        let mut file_content = load_file_to_string(&p)?;
        file_content = file_content.replace("\n",""); //github adds new line to the end of csv. its a workaround
        let splitted = file_content.split(",");
        //TODO limit number of votes! and check parsing
        for key in splitted {
            *hm.entry(key.to_string()).or_insert(0) += 1;
        }
    }
    Ok(hm)
}


fn get_all_vote_file_paths() -> Result<Vec<String>> {
    get_files_paths_in_directory("votes/")
}

fn get_files_paths_in_directory(path: &str) -> Result<Vec<String>> {
    let mut paths = Vec::new();
    let path = Path::new(path);
    for p in path.read_dir()? {
        let unwrapped = p?;
        let as_string = unwrapped
            .path()
            .to_str()
            .expect("failed to get path")
            .to_string();
        println!("file found: {:?}", &as_string);
        paths.push(as_string);
    }
    Ok(paths)
}

fn generate_votes_page(votes: HashMap<String, u32>, ideas: HashMap<String, Idea>) -> Result<()> {
    let mut s = slime::Slime::new();
    let mut jsondata = s.load_data("index", slime::DataFormat::Json)
        .expect("failed to parse json data for template"); //todo
    let ideas_as_json = ideas_to_json(votes, ideas)
        .chain_err(|| "failed to change votes hashmap to json object")?;
    jsondata["ideas"] = ideas_as_json;
    s.add("index", &jsondata, "index");

    s.run().expect("failed to generate pages with slime");
    Ok(())
}

fn ideas_to_json(votes: HashMap<String, u32>, ideas: HashMap<String, Idea>) -> Result<SerdeJson> {
    let mut as_vector: Vec<(String, u32)> =
        votes.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
    as_vector.sort_by_key(|t| {
                              let &(_, v) = t;
                              v
                          });
    as_vector.reverse();
    let v: Vec<SerdeJson> = as_vector
        .into_iter()
        .map(|(k, v)| {
            let idea_ref: &Idea =
                ideas
                    .get(&k)
                    .expect(&format!("failed to find idea with this name! name: {}", &k));
            let idea_cloned: Idea = (*idea_ref).clone();

            let wrapper = IdeaWrapper {
                name: k.clone(),
                votes: v,
                idea: idea_cloned,
            };
            json!(wrapper)
        })
        .collect();

    Ok(json!(v))
}

fn get_ideas() -> Result<HashMap<String, Idea>> {
    let mut v = HashMap::new();
    let paths = get_files_paths_in_directory("ideas/")?;
    for p in paths {
        let content = load_file_to_string(&p)?;
        let idea: Idea = toml::from_str(&content).expect("failed to parse toml file");
        let file_name = get_file_name(&p);
        v.insert(file_name, idea);
    }
    Ok(v)
}


fn load_file_to_string(path: &str) -> Result<String> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

fn get_file_name(path: &str) -> String {
    let splitted = path.split("/");
    let last = splitted.last().unwrap();
    let mut without_extension = last.split(".");
    without_extension.next().unwrap().to_string()
}