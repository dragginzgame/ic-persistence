//! Release effects are substituted; native Git checks use disposable indexes.
//! No fixture commits, tags, pushes or live publication occur.

use super::*;
use ic_host_process::tool::{
    CompletedExecution, OutputLimit, OutputLimits, ToolError, capture_command,
};
use std::{cell::RefCell, process::Command};

mod build;
mod files;
mod git;
mod registry;
use build::{BuildEffects, BuildFault};
use files::{FileEffects, FileFault};
use git::{GitFault, GitState};
use registry::{RegistryEffects, RegistryFault};

mod fixtures;
use fixtures::*;

mod archives;
mod configuration;
mod preparation;
mod process_io;
mod publication;
mod recovery;
mod source_admission;
