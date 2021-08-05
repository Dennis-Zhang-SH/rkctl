use regex::Regex;
use std::{
    io::{stdin},
    process::{Command, Stdio},
};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
enum Action {
    Get(Resource),
    Delete(Resource),
}

impl From<&Action> for String {
    fn from(val: &Action) -> Self {
        match val {
            Action::Get(_) => String::from("get"),
            Action::Delete(_) => String::from("delete"),
        }
    }
}

impl Action {
    fn unwrap(&self) -> &Resource {
        match &self {
            Action::Get(r) => r,
            Action::Delete(r) => r,
        }
    }
}

#[derive(Debug, StructOpt)]
enum Resource {
    Pod(DetailedResource),
    Pods(DetailedResource),
    Service(DetailedResource),
    Svc(DetailedResource),
}

impl From<&Resource> for String {
    fn from(val: &Resource) -> Self {
        match val {
            Resource::Pod(_) | Resource::Pods(_) => String::from("pod"),
            _ => String::from("svc"),
        }
    }
}

impl Resource {
    fn unwrap(&self) -> &DetailedResource {
        match &self {
            Resource::Pod(d) => d,
            Resource::Pods(d) => d,
            Resource::Service(d) => d,
            Resource::Svc(d) => d,
        }
    }
}

#[derive(Debug, Clone, StructOpt)]
struct DetailedResource {
    resource_name: String,
}

#[derive(Debug, StructOpt)]
#[structopt(name = "rkctl", about = "An regex version of kubectl.")]
struct RawCommand {
    #[structopt(subcommand)]
    action: Action,
    #[structopt(short, long, default_value = "default")]
    namespace: String,
}

impl RawCommand {
    fn new() -> Self {
        RawCommand::from_args()
    }
}

impl Into<ParsedCommand> for RawCommand {
    fn into(self) -> ParsedCommand {
        ParsedCommand {
            raw_command: self,
            regex: None,
        }
    }
}

struct ParsedCommand {
    raw_command: RawCommand,
    regex: Option<Regex>,
}

impl ParsedCommand {
    fn start(&mut self) {
        let d = self.raw_command.action.unwrap().unwrap().clone();
        self.check_regex(&d);
        let r = self.get_all_matched_resources();
        match self.raw_command.action {
            Action::Get(_) => return,
            _ => {
                println!("Are you sure you want to delete all these resources? [y/N]");
                let stdin = stdin();
                let mut yes_or_no = String::new();
                stdin.read_line(&mut yes_or_no).unwrap();
                if yes_or_no.starts_with("y") || yes_or_no.starts_with("Y") {
                    for name in r {
                        println!(
                            "executing command kubectl {} {} {} -n {} now",
                            String::from(&self.raw_command.action),
                            String::from(self.raw_command.action.unwrap()),
                            &name,
                            &self.raw_command.namespace
                        );
                        let _result = Command::new("kubectl")
                            .args([
                                String::from(&self.raw_command.action),
                                String::from(self.raw_command.action.unwrap()),
                                name,
                                "-n".to_owned(),
                                self.raw_command.namespace.clone()
                            ])
                            .stdout(Stdio::inherit())
                            .output()
                            .expect("failed to excute command");
                    }
                } else {
                    println!("exiting now...")
                }
            }
        }
    }

    fn check_regex(&mut self, d: &DetailedResource) {
        match &self.regex {
            Some(_) => {}
            None => match Regex::new(&d.resource_name) {
                Ok(r) => self.regex = Some(r),
                _ => {}
            },
        }
    }

    fn get_all_matched_resources(&mut self) -> Vec<String> {
        let mut cmd = Command::new("kubectl");
        cmd.args([
            String::from(&self.raw_command.action),
            String::from(self.raw_command.action.unwrap()),
        ]);
        cmd.args(["-n", &self.raw_command.namespace]);

        if let Some(r) = &self.regex {
            let mut all_res_cmd = Command::new("kubectl");
            all_res_cmd.args([
                "get",
                String::from(self.raw_command.action.unwrap()).as_str(),
            ]);
            all_res_cmd.args(["-n", &self.raw_command.namespace]);
            let all_res = all_res_cmd.output().expect("failed to get all resources");
            let lines: Vec<_> = all_res.stdout.split(|v| v == &b'\n').collect();
            let mut lines = lines.into_iter();
            println!(
                "{}",
                String::from_utf8(lines.next().unwrap_or(b"").to_vec()).unwrap()
            );
            let mut matched_result = Vec::new();
            for line in lines {
                let l = line.to_owned();
                let name = line
                    .clone()
                    .split(|x| x == &b' ')
                    .collect::<Vec<_>>()
                    .swap_remove(0);
                let name = String::from_utf8(name.to_vec()).unwrap();
                if r.is_match(&name) {
                    matched_result.push(name);
                    println!("{}", String::from_utf8(l).unwrap());
                }
            }
            matched_result
        } else {
            cmd.arg(&self.raw_command.action.unwrap().unwrap().resource_name);
            let res = cmd.output().expect("failed to get all resources");
            println!("{}", String::from_utf8(res.stdout).unwrap());
            vec![self
                .raw_command
                .action
                .unwrap()
                .unwrap()
                .resource_name
                .clone()]
        }
    }
}

fn main() {
    let mut rkcmd: ParsedCommand = RawCommand::new().into();
    rkcmd.start();
}
