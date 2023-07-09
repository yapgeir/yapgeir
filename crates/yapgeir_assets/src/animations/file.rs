use super::{Animation, AnimationKind, AnimationSequence};
use crate::atlas::Atlas;
use anyhow::Result;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimationData {
    pub name: String,
    pub kind: AnimationKind,
    pub speed: f32,
    pub sprite: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimationSequenceData {
    pub name: String,
    pub animations: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimationFile {
    pub animations: Vec<AnimationData>,
    pub sequences: Vec<AnimationSequenceData>,
}

impl AnimationFile {
    pub fn decode(yaml: &str) -> Result<AnimationFile> {
        Ok(serde_yaml::from_str(yaml)?)
    }

    pub fn to_sequence_map(&self, atlas: &Atlas) -> HashMap<String, AnimationSequence> {
        let mut implicit_sequences = self
            .animations
            .iter()
            .map(|a| {
                let indexes = atlas.frame_tags.get(&a.sprite).cloned().unwrap_or(0..=255);
                let frames = indexes
                    .into_iter()
                    .map(|i| format!("{}_{}", a.sprite, i))
                    .map(|i| atlas.sprites.get(&i))
                    .take_while(|v| v.is_some())
                    .flatten()
                    .map(|v| v.sub_texture.clone())
                    .collect();

                (
                    a.name.clone(),
                    AnimationSequence(vec![Animation {
                        frames,
                        frame_time: a.speed,
                        kind: a.kind,
                    }]),
                )
            })
            .collect::<HashMap<_, _>>();

        let explicit_sequences = self
            .sequences
            .iter()
            .map(|s| {
                (
                    s.name.clone(),
                    AnimationSequence(
                        s.animations
                            .iter()
                            .map(|a| implicit_sequences[a].0[0].clone())
                            .collect(),
                    ),
                )
            })
            .collect::<HashMap<_, _>>();

        implicit_sequences.extend(explicit_sequences);
        implicit_sequences
    }
}
