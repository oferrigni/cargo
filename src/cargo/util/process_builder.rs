use std::collections::HashMap;
use std::env;
use std::ffi::{OsString, AsOsStr};
use std::fmt;
use std::path::Path;
use std::process::{Command, Output};

use util::{CargoResult, ProcessError, process_error};

#[derive(Clone, PartialEq, Debug)]
pub struct ProcessBuilder {
    program: OsString,
    args: Vec<OsString>,
    env: HashMap<String, Option<OsString>>,
    cwd: OsString,
}

impl fmt::Display for ProcessBuilder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "`{}", self.program.to_string_lossy()));

        for arg in self.args.iter() {
            try!(write!(f, " {}", arg.to_string_lossy()));
        }

        write!(f, "`")
    }
}

impl ProcessBuilder {
    pub fn arg<T: AsOsStr + ?Sized>(&mut self, arg: &T) -> &mut ProcessBuilder {
        self.args.push(arg.as_os_str().to_os_string());
        self
    }

    pub fn args<T: AsOsStr>(&mut self, arguments: &[T]) -> &mut ProcessBuilder {
        self.args.extend(arguments.iter().map(|t| {
            t.as_os_str().to_os_string()
        }));
        self
    }

    pub fn cwd<T: AsOsStr + ?Sized>(&mut self, path: &T) -> &mut ProcessBuilder {
        self.cwd = path.as_os_str().to_os_string();
        self
    }

    pub fn env<T: AsOsStr + ?Sized>(&mut self, key: &str,
                                    val: &T) -> &mut ProcessBuilder {
        self.env.insert(key.to_string(), Some(val.as_os_str().to_os_string()));
        self
    }

    pub fn env_remove(&mut self, key: &str) -> &mut ProcessBuilder {
        self.env.insert(key.to_string(), None);
        self
    }

    pub fn get_args(&self) -> &[OsString] {
        &self.args
    }
    pub fn get_cwd(&self) -> &Path { Path::new(&self.cwd) }

    pub fn get_env(&self, var: &str) -> Option<OsString> {
        self.env.get(var).cloned().or_else(|| Some(env::var_os(var)))
            .and_then(|s| s)
    }

    pub fn get_envs(&self) -> &HashMap<String, Option<OsString>> { &self.env }

    pub fn exec(&self) -> Result<(), ProcessError> {
        let mut command = self.build_command();
        let exit = try!(command.status().map_err(|e| {
            process_error(&format!("Could not execute process `{}`",
                                   self.debug_string()),
                          Some(e), None, None)
        }));

        if exit.success() {
            Ok(())
        } else {
            Err(process_error(&format!("Process didn't exit successfully: `{}`",
                                       self.debug_string()),
                              None, Some(&exit), None))
        }
    }

    pub fn exec_with_output(&self) -> Result<Output, ProcessError> {
        let mut command = self.build_command();

        let output = try!(command.output().map_err(|e| {
            process_error(&format!("Could not execute process `{}`",
                               self.debug_string()),
                          Some(e), None, None)
        }));

        if output.status.success() {
            Ok(output)
        } else {
            Err(process_error(&format!("Process didn't exit successfully: `{}`",
                                       self.debug_string()),
                              None, Some(&output.status), Some(&output)))
        }
    }

    pub fn build_command(&self) -> Command {
        let mut command = Command::new(&self.program);
        command.current_dir(&self.cwd);
        for arg in self.args.iter() {
            command.arg(arg);
        }
        for (k, v) in self.env.iter() {
            match *v {
                Some(ref v) => { command.env(k, v); }
                None => { command.env_remove(k); }
            }
        }
        command
    }

    fn debug_string(&self) -> String {
        let mut program = format!("{}", self.program.to_string_lossy());
        for arg in self.args.iter() {
            program.push(' ');
            program.push_str(&format!("{}", arg.to_string_lossy()));
        }
        program
    }
}

pub fn process<T: AsOsStr + ?Sized>(cmd: &T) -> CargoResult<ProcessBuilder> {
    Ok(ProcessBuilder {
        program: cmd.as_os_str().to_os_string(),
        args: Vec::new(),
        cwd: try!(env::current_dir()).as_os_str().to_os_string(),
        env: HashMap::new(),
    })
}
