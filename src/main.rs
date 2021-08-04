use bytes::Bytes;
use regex::Regex;
use std::process::Command;
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
    #[structopt(short, long)]
    namespace: Option<String>,
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
            result: None,
        }
    }
}

struct ParsedCommand {
    raw_command: RawCommand,
    regex: Option<Regex>,
    result: Option<Bytes>,
}

impl ParsedCommand {
    fn start(&mut self) {
        let d = self.raw_command.action.unwrap().unwrap().clone();
        self.check_regex(&d);
        let mut cmd = Command::new("kubectl");
        cmd.args([
            String::from(&self.raw_command.action),
            String::from(self.raw_command.action.unwrap()),
        ]);
        if let Some(namespace) = &self.raw_command.namespace {
            cmd.args(["-n", namespace]);
        }
        if let Some(r) = &self.regex {
            let mut all_res_cmd = Command::new("kubectl");
            all_res_cmd.args([
                "get",
                String::from(self.raw_command.action.unwrap()).as_str(),
            ]);
            if let Some(namespace) = &self.raw_command.namespace {
                all_res_cmd.args(["-n", namespace]);
            }
            let all_res = all_res_cmd.output().expect("failed to get all resources");
            let lines: Vec<_> = all_res.stdout.split(|v| v == &b'\n').collect();
            let mut lines = lines.into_iter();
            println!("{:?}", lines.next().unwrap_or(b""));
            for line in lines {
                
            }
        } else {
            cmd.arg(d.resource_name);
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
}

fn main() {
    let rkcmd: ParsedCommand = RawCommand::new().into();
}
