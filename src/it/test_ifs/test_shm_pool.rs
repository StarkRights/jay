use {
    crate::{
        format::Format,
        it::{
            test_error::TestError, test_ifs::test_shm_buffer::TestShmBuffer, test_mem::TestMem,
            test_object::TestObject, test_transport::TestTransport,
        },
        utils::clonecell::CloneCell,
        wire::{wl_shm_pool::*, WlShmPoolId},
    },
    std::{cell::Cell, rc::Rc},
};

pub struct TestShmPool {
    pub id: WlShmPoolId,
    pub tran: Rc<TestTransport>,
    pub mem: CloneCell<Rc<TestMem>>,
    pub destroyed: Cell<bool>,
}

impl TestShmPool {
    pub fn create_buffer(
        &self,
        offset: i32,
        width: i32,
        height: i32,
        stride: i32,
        format: &Format,
    ) -> Result<Rc<TestShmBuffer>, TestError> {
        let size = (height * stride) as usize;
        let start = offset as usize;
        let end = start + size;
        let mem = self.mem.get();
        if end > mem.len() {
            bail!("Out-of-bounds buffer");
        }
        let buffer = Rc::new(TestShmBuffer {
            id: self.tran.id(),
            tran: self.tran.clone(),
            range: start..end,
            mem,
            released: Cell::new(true),
            destroyed: Cell::new(false),
        });
        self.tran.add_obj(buffer.clone())?;
        self.tran.send(CreateBuffer {
            self_id: self.id,
            id: buffer.id,
            offset,
            width,
            height,
            stride,
            format: format.wl_id.unwrap_or(format.drm),
        });
        Ok(buffer)
    }

    pub fn resize(&self, size: usize) -> Result<(), TestError> {
        let mem = self.mem.get().grow(size)?;
        self.mem.set(mem);
        self.tran.send(Resize {
            self_id: self.id,
            size: size as _,
        });
        Ok(())
    }

    pub fn destroy(&self) {
        if self.destroyed.replace(true) {
            return;
        }
        self.tran.send(Destroy { self_id: self.id });
    }
}

impl Drop for TestShmPool {
    fn drop(&mut self) {
        self.destroy()
    }
}

test_object! {
    TestShmPool, WlShmPool;
}

impl TestObject for TestShmPool {}