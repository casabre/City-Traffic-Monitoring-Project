pub trait Runner<'a> {
    fn start(&self);
    fn stop(&self);
}

pub trait MutableRunner<'a> {
    fn start(&mut self);
    fn stop(&mut self);
}
