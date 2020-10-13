use crate::{Drawable, GfxContext, InstanceRaw, SpriteBatch, SpriteBatchBuilder, Texture};
use std::path::Path;
use wgpu::RenderPass;

pub struct MultiSpriteBatchBuilder {
    sbs: Vec<SpriteBatchBuilder>,
}

#[derive(Clone)]
pub struct MultiSpriteBatch {
    pub sbs: Vec<SpriteBatch>,
}

impl MultiSpriteBatchBuilder {
    pub fn from_paths(ctx: &GfxContext, paths: &[impl AsRef<Path>]) -> Self {
        Self {
            sbs: paths
                .iter()
                .map(move |path| SpriteBatchBuilder::from_path(ctx, path))
                .collect(),
        }
    }

    pub fn new(texs: Vec<Texture>) -> Self {
        Self {
            sbs: texs.into_iter().map(SpriteBatchBuilder::new).collect(),
        }
    }

    pub fn n_texs(&self) -> usize {
        self.sbs.len()
    }

    pub fn build(&self, gfx: &GfxContext) -> Option<MultiSpriteBatch> {
        let sb: Option<Vec<SpriteBatch>> = self.sbs.iter().map(|sb| sb.build(gfx)).collect();
        sb.map(|v| MultiSpriteBatch { sbs: v })
    }

    pub fn clear(&mut self) {
        for sbb in &mut self.sbs {
            sbb.instances.clear();
        }
    }

    pub fn push(&mut self, id: usize, instance: InstanceRaw) {
        self.sbs[id].instances.push(instance);
    }
}

impl Drawable for MultiSpriteBatch {
    fn create_pipeline(_gfx: &GfxContext) -> super::PreparedPipeline {
        unimplemented!()
    }

    fn draw<'a>(&'a self, gfx: &'a GfxContext, rp: &mut RenderPass<'a>) {
        for v in &self.sbs {
            v.draw(gfx, rp);
        }
    }
}
