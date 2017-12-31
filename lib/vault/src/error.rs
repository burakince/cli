use serde_yaml;
use std::io;
use std::path::{Path, PathBuf};
use std::fmt;
use failure::Fail;
use gpgme;
use failure;

#[derive(Debug, Fail)]
#[fail(display = "The content was not encrypted for you.")]
pub struct DecryptionError {
    #[cause] pub cause: gpgme::Error,
}

impl DecryptionError {
    pub fn caused_by(err: gpgme::Error, alternative_text: &'static str) -> failure::Error {
        if err.code() == gpgme::Error::NO_SECKEY.code() {
            failure::Error::from(DecryptionError { cause: err })
        } else {
            err.context(alternative_text).into()
        }
    }
}

#[derive(Debug, Fail)]
#[fail(display = "At least one recipient you try to encrypt for is untrusted. \
                  Consider (locally) signing the key with `gpg --sign-key <recipient>` \
                  or ultimately trusting them.")]
pub struct EncryptionError {
    #[cause] pub cause: gpgme::Error,
}

impl EncryptionError {
    pub fn caused_by(err: gpgme::Error, alternative_text: String) -> failure::Error {
        if err.code() == gpgme::Error::UNUSABLE_PUBKEY.code() {
            failure::Error::from(EncryptionError { cause: err })
        } else {
            err.context(alternative_text).into()
        }
    }
}

#[derive(Debug, Fail)]
pub enum VaultError {
    ConfigurationFileExists(PathBuf),
    ReadFile {
        #[cause] cause: io::Error,
        path: PathBuf,
    },
    WriteFile {
        #[cause] cause: io::Error,
        path: PathBuf,
    },
    Deserialization {
        #[cause] cause: serde_yaml::Error,
        path: PathBuf,
    },
    Serialization {
        #[cause] cause: serde_yaml::Error,
        path: PathBuf,
    },
}

impl fmt::Display for VaultError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::VaultError::*;
        match *self {
            ConfigurationFileExists(ref path) => writeln!(
                f,
                "Cannot overwrite vault configuration file as it already exists at '{}'",
                path.display()
            ),
            Serialization { ref path, .. } => writeln!(
                f,
                "Failed to serialize vault configuration file at '{}'",
                path.display()
            ),
            Deserialization { ref path, .. } => writeln!(
                f,
                "Failed to deserialize vault configuration file at '{}'",
                path.display()
            ),
            WriteFile { ref path, .. } => writeln!(
                f,
                "Failed to write vault configuration file at '{}'",
                path.display()
            ),
            ReadFile { ref path, .. } => writeln!(
                f,
                "Failed to read vault configuration file at '{}'",
                path.display()
            ),
        }
    }
}

pub enum IOMode {
    Read,
    Write,
}

impl VaultError {
    pub fn from_io_err(cause: io::Error, path: &Path, mode: &IOMode) -> Self {
        match *mode {
            IOMode::Write => VaultError::WriteFile {
                cause,
                path: path.to_owned(),
            },
            IOMode::Read => VaultError::ReadFile {
                cause,
                path: path.to_owned(),
            },
        }
    }
}

pub trait FailExt: Fail {
    fn first_cause_of<T: Fail>(&self) -> Option<&T>;
}

impl<F> FailExt for F
where
    F: Fail,
{
    fn first_cause_of<T: Fail>(&self) -> Option<&T> {
        self.causes().filter_map(|c| c.downcast_ref::<T>()).next()
    }
}

// https://github.com/withoutboats/failure/pull/124
pub fn first_cause_of_type<T: Fail>(root: &failure::Error) -> Option<&T> {
    root.causes().filter_map(|c| c.downcast_ref::<T>()).next()
}
