use crate::cache::WorldInfoCache;
use crate::*;

// This is a first attempt at sharing common term building functionality
// between Filters, Queries, and Systems
//
pub trait TermBuilder: Sized {
    fn world(&mut self) -> *mut ecs_world_t;
    fn filter_desc(&mut self) -> &mut ecs_filter_desc_t;
    fn current_term(&mut self) -> &mut ecs_term_t;
    fn next_term(&mut self);

    // This is not working right
    // fn expr(mut self, expr: &str) -> Self {
    // 	let filter = self.filter_desc();
    // 	if expr.len() > 0 {
    // 		let expr_c_str = std::ffi::CString::new(expr).unwrap();
    // 		filter.expr = expr_c_str.as_ptr() as *const i8;
    // 	}
    // 	self
    // }

    fn term<A: Component>(mut self) -> Self {
        let world_raw = self.world();
        let term = self.current_term();

        term.id = WorldInfoCache::get_component_id_for_type::<A>(world_raw)
            .expect("Component type not registered!");

        self.next_term();
        self
    }

    fn without<A: Component>(mut self) -> Self {
        let world_raw = self.world();
        let term = self.current_term();

        term.id = WorldInfoCache::get_component_id_for_type::<A>(world_raw)
            .expect("Component type not registered!");
        term.oper = ecs_oper_kind_t_EcsNot;

        self.next_term();
        self
    }

    fn term_dynamic(mut self, comp_id: EntityId) -> Self {
        // TODO - validate that the comp_id passed is valid
        let term = self.current_term();
        term.id = comp_id;
        self.next_term();
        self
    }

    fn with_components<'c, G: ComponentGroup<'c>>(mut self) -> Self {
        G::populate(&mut self);
        self
    }
}
