use super::GameMode;
use std::collections::HashSet;
use stupids3::{get_obj, put, StupidS3Error};

pub struct GameDiff {
    pub added: HashSet<GameMode>,
    pub removed: HashSet<GameMode>,
}

pub trait ArcadeState {
    fn previous_modes(&self) -> Result<Vec<GameMode>, failure::Error>;
    fn set_modes<'a>(
        &mut self,
        modes: impl Iterator<Item = &'a GameMode>,
    ) -> Result<(), failure::Error>;
    fn mode_diff<'a>(
        &'a self,
        modes: impl Iterator<Item = &'a GameMode>,
    ) -> Result<GameDiff, failure::Error> {
        let next = modes.cloned().collect::<HashSet<_>>();
        let prev = self.previous_modes()?.into_iter().collect::<HashSet<_>>();
        Ok(GameDiff {
            added: next.difference(&prev).cloned().collect(),
            removed: prev.difference(&next).cloned().collect(),
        })
    }
}

pub struct S3State {
    pub bucket: String,
    pub keyname: String,
}

impl ArcadeState for S3State {
    fn previous_modes(&self) -> Result<Vec<GameMode>, failure::Error> {
        match get_obj::<Vec<GameMode>, _, _>(&self.bucket, &self.keyname) {
            Ok(r) => Ok(r),
            Err(s3_err) => match s3_err {
                StupidS3Error::UnknownError { .. } => Err(s3_err.into()),
                _ => {
                    warn!("got error: {:#?}", s3_err);
                    Ok(Vec::new())
                }
            },
        }
    }
    fn set_modes<'a>(
        &mut self,
        modes: impl Iterator<Item = &'a GameMode>,
    ) -> Result<(), failure::Error> {
        put(&self.bucket, &self.keyname, &modes.collect::<Vec<_>>())?;
        Ok(())
    }
}
