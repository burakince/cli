extern crate s3_types;

use conv::TryInto;
use s3_types::{CreateMode, ExtractionContext, VaultCommand, VaultContext};
use clap::{App, ArgMatches, Shell};
use failure::{err_msg, Error, ResultExt};
use std::io::stdout;
use std::path::Path;
use std::str::FromStr;
use std::convert::Into;
use std::ffi::OsStr;
use cli::CLI;

fn required_os_arg<'a, T>(args: &'a ArgMatches, name: &'static str) -> Result<T, Error>
where
    T: From<&'a OsStr>,
{
    match args.value_of_os(name).map(Into::into) {
        Some(t) => Ok(t),
        None => Err(format_err!(
            "BUG: expected clap argument '{}' to be set",
            name
        )),
    }
}

fn required_arg<T>(args: &ArgMatches, name: &'static str) -> Result<T, Error>
where
    T: FromStr,
    <T as FromStr>::Err: 'static + ::std::error::Error + Send + Sync,
{
    match args.value_of(name).map(FromStr::from_str) {
        Some(Ok(t)) => Ok(t),
        Some(Err(e)) => Err(e.into()),
        None => Err(format_err!(
            "BUG: expected clap argument '{}' to be set",
            name
        )),
    }
}

pub fn vault_context_from(args: &ArgMatches) -> Result<VaultContext, Error> {
    Ok(VaultContext {
        vault_path: required_os_arg(args, "config-file")?,
        vault_id: required_arg(args, "vault-id")?,
        command: VaultCommand::List,
    })
}

pub fn vault_recipients_add(ctx: VaultContext, args: &ArgMatches) -> Result<VaultContext, Error> {
    Ok(VaultContext {
        command: VaultCommand::RecipientsAdd {
            gpg_key_ids: args.values_of("gpg-key-id")
                .expect("Clap to assure this is a required arg")
                .map(Into::into)
                .collect(),
        },
        ..ctx
    })
}

pub fn vault_resource_show(ctx: VaultContext, args: &ArgMatches) -> Result<VaultContext, Error> {
    Ok(VaultContext {
        command: VaultCommand::ResourceShow {
            spec: required_os_arg(args, "path")?,
        },
        ..ctx
    })
}

pub fn vault_resource_edit(ctx: VaultContext, args: &ArgMatches) -> Result<VaultContext, Error> {
    Ok(VaultContext {
        command: VaultCommand::ResourceEdit {
            spec: required_os_arg(args, "path")?,
            editor: required_os_arg(args, "editor")?,
            mode: match args.is_present("no-create") {
                true => CreateMode::NoCreate,
                false => CreateMode::Create,
            },
        },
        ..ctx
    })
}

pub fn vault_resource_add_from(ctx: VaultContext, args: &ArgMatches) -> Result<VaultContext, Error> {
    Ok(VaultContext {
        command: VaultCommand::ResourceAdd {
            specs: match args.values_of("spec") {
                Some(v) => v.map(|s| s.try_into()).collect::<Result<_, _>>()?,
                None => Vec::new(),
            },
        },
        ..ctx
    })
}

pub fn vault_init_from(ctx: VaultContext, args: &ArgMatches) -> Result<VaultContext, Error> {
    Ok(VaultContext {
        command: VaultCommand::Init {
            recipients_file: required_os_arg(args, "recipients-file-path")?,
            gpg_keys_dir: required_os_arg(args, "gpg-keys-dir")?,
            gpg_key_ids: match args.values_of("gpg-key-id") {
                Some(v) => v.map(Into::into).collect(),
                None => Vec::new(),
            },
        },
        ..ctx
    })
}

pub fn extraction_context_from(args: &ArgMatches) -> Result<ExtractionContext, Error> {
    Ok(ExtractionContext {
        file_path: required_os_arg(args, "file")?,
    })
}

pub fn generate_completions(mut app: App, args: &ArgMatches) -> Result<String, Error> {
    let shell = args.value_of("shell")
        .ok_or_else(|| err_msg("expected 'shell' argument"))
        .map(|s| {
            Path::new(s)
                .file_name()
                .map(|f| f.to_str().expect("os-string to str conversion failed"))
                .unwrap_or(s)
        })
        .and_then(|s| {
            Shell::from_str(s)
                .map_err(err_msg)
                .context(format!("The shell '{}' is unsupported", s))
                .map_err(Into::into)
        })?;
    app.gen_completions_to(CLI::name(), shell, &mut stdout());
    Ok(String::new())
}