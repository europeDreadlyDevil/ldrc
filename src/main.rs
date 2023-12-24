use std::{env, fs};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

extern crate clap;
use clap::{Parser};
use fs_extra::dir::copy;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    path: String,
}


fn main() {
    let args = Args::parse();
    let temp_path = env::temp_dir().join("ldrc-temp");
    match fs::create_dir(&temp_path) {
        Ok(_) => {
            let path = args.path.replace(".", env::current_dir().unwrap().to_str().unwrap());
            let options = fs_extra::dir::CopyOptions {
                content_only: true,
                overwrite: true,
                ..Default::default()
            };

            match copy(&path, &temp_path, &options) {
                Ok(_) => {
                    let compose_content = format!("{}", "services:
  linux:
    image: europedreadlydevil/ldrc
    network_mode: host
    volumes:
      - ./:/project/
      - ./target:/project/target");
                    println!("{compose_content}");
                    let mut docker_compose_file = File::create(&temp_path.join("docker-compose.yml")).unwrap();
                    docker_compose_file.write_all(compose_content.as_bytes()).unwrap();
                    env::set_current_dir(&temp_path).unwrap();
                    println!("{:?}", env::current_dir().unwrap());
                    let mut build_command = Command::new("docker-compose")
                        .arg("build")
                        .spawn()
                        .expect("Failed to execute docker-compose build");

                    let build_status = build_command.wait().expect("Failed to wait for docker-compose build");

                    if build_status.success() {
                        println!("Docker-compose build executed successfully");

                        let mut up_command = Command::new("docker-compose")
                            .arg("up")
                            .spawn()
                            .expect("Failed to execute docker-compose up");

                        up_command.wait().expect("Failed to wait for docker-compose up");

                        let mut remove_command = Command::new("docker-compose")
                            .arg("down")
                            .spawn()
                            .expect("Failed to execute docker-compose up");

                        remove_command.wait().expect("Failed to wait for docker rm ldrc");
                    }
                    if let Err(err) = copy(env::current_dir().unwrap().join("target/debug"), PathBuf::from(path).join("target/linux"), &options) {
                        eprintln!("Error when copying a directory: {}", err)
                    }
                    env::set_current_dir(&temp_path.parent().unwrap()).unwrap();
                    if let Err(e) = fs::remove_dir_all(&temp_path) { eprintln!("{e}"); }
                },
                Err(err) => eprintln!("Error when copying a directory: {}", err),
                }
            }
            Err(e) => { eprintln!("{e}") }
        }
}
