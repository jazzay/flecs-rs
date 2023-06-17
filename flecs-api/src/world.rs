use std::alloc::Layout;
use std::any::TypeId;
use std::mem::MaybeUninit;

use crate::cache::WorldInfoCache;
use crate::*;

pub struct World {
    world: *mut ecs_world_t,
    owned: bool,
}

impl World {
    /// Creates a new Flecs World instance
    pub fn new() -> Self {
        let world = unsafe { ecs_init() };
        WorldInfoCache::insert(world);
        let mut w = Self { world, owned: true };
        w.init_builtin_components();
        w
    }

    pub(crate) fn new_from(world: *mut ecs_world_t) -> Self {
        Self {
            world,
            owned: false,
        }
    }

    fn init_builtin_components(&mut self) {
        // TODO: Get access to these components, and determine if component_named
        // is sufficient or if these need to be paths?
        // self.component_named::<Component>("flecs::core::Component");
        // self.component_named::<Identifier>("flecs::core::Identifier");
        // self.component_named::<Poly>("flecs::core::Poly");

        // TODO - register all the module components as well
        // #   ifdef FLECS_SYSTEM
        // 	_::system_init(*this);
        // #   endif
        // #   ifdef FLECS_TIMER
        // 	_::timer_init(*this);
        // #   endif
        // #   ifdef FLECS_DOC
        // 	doc::_::init(*this);
        // #   endif
        // #   ifdef FLECS_REST
        // 	rest::_::init(*this);
        // #   endif
        // #   ifdef FLECS_META
        // 	meta::_::init(*this);
        // #   endif
    }

    pub fn raw(&self) -> *mut ecs_world_t {
        self.world
    }

    /// Deletes and recreates the world
    pub fn reset(&mut self) {
        assert!(self.owned);
        unsafe {
            ecs_fini(self.world);
            self.world = ecs_init();
            WorldInfoCache::insert(self.world);
        }
        self.init_builtin_components();
    }

    pub fn entity(&self) -> Entity {
        let entity = unsafe { ecs_new_id(self.world) };
        Entity::new(self.world, entity)
    }

    pub fn prefab(&self, name: &str) -> Entity {
        unsafe {
            let entity = ecs_new_id(self.world);
            Entity::new(self.world, entity)
                .named(name)
                .add_id(EcsPrefab)
        }
    }

    pub fn progress(&self, delta_time: f32) -> bool {
        unsafe { ecs_progress(self.world, delta_time) }
    }

    /// Get current frame delta time
    pub fn delta_time(&self) -> f32 {
        unsafe {
            let stats = ecs_get_world_info(self.world).as_ref().unwrap();
            stats.delta_time
        }
    }

    /// Get current tick (in frames)
    pub fn tick(&self) -> i64 {
        unsafe {
            let stats = ecs_get_world_info(self.world).as_ref().unwrap();
            stats.frame_count_total
        }
    }

    /// Get current simulation time
    pub fn time(&self) -> f32 {
        unsafe {
            let stats = ecs_get_world_info(self.world).as_ref().unwrap();
            stats.world_time_total
        }
    }

    /** Signal application should quit.
     * After calling this operation, the next call to progress() returns false.
     */
    pub fn quit(&self) {
        unsafe { ecs_quit(self.world) }
    }

    /** Test if quit() has been called.
     */
    fn should_quit(&self) -> bool {
        unsafe { ecs_should_quit(self.world) }
    }

    /** Begin frame.
     * When an application does not use progress() to control the main loop, it
     * can still use Flecs features such as FPS limiting and time measurements.
     * This operation needs to be invoked whenever a new frame is about to get
     * processed.
     *
     * Calls to frame_begin must always be followed by frame_end.
     *
     * The function accepts a delta_time parameter, which will get passed to
     * systems. This value is also used to compute the amount of time the
     * function needs to sleep to ensure it does not exceed the target_fps, when
     * it is set. When 0 is provided for delta_time, the time will be measured.
     *
     * This function should only be ran from the main thread.
     *
     * @param delta_time Time elapsed since the last frame.
     * @return The provided delta_time, or measured time if 0 was provided.
     */
    fn frame_begin(&self, delta_time: f32) -> f32 {
        unsafe { ecs_frame_begin(self.world, delta_time) }
    }

    /** End frame.
     * This operation must be called at the end of the frame, and always after
     * ecs_frame_begin.
     *
     * This function should only be ran from the main thread.
     */
    fn frame_end(&self) {
        unsafe {
            ecs_frame_end(self.world);
        }
    }

    /** Begin staging.
     * When an application does not use ecs_progress to control the main loop, it
     * can still use Flecs features such as the defer queue. When an application
     * needs to stage changes, it needs to call this function after ecs_frame_begin.
     * A call to ecs_readonly_begin must be followed by a call to ecs_readonly_end.
     *
     * When staging is enabled, modifications to entities are stored to a stage.
     * This ensures that arrays are not modified while iterating. Modifications are
     * merged back to the "main stage" when ecs_readonly_end is invoked.
     *
     * While the world is in staging mode, no structural changes (add/remove/...)
     * can be made to the world itself. Operations must be executed on a stage
     * instead (see ecs_get_stage).
     *
     * This function should only be ran from the main thread.
     *
     * @return Whether world is currently staged.
     */
    fn readonly_begin(&self) -> bool {
        unsafe { ecs_readonly_begin(self.world) }
    }

    /** End staging.
     * Leaves staging mode. After this operation the world may be directly mutated
     * again. By default this operation also merges data back into the world, unless
     * automerging was disabled explicitly.
     *
     * This function should only be ran from the main thread.
     */
    fn readonly_end(&self) {
        unsafe {
            ecs_readonly_end(self.world);
        }
    }

    /** Defer operations until end of frame.
     * When this operation is invoked while iterating, operations inbetween the
     * defer_begin and defer_end operations are executed at the end of the frame.
     *
     * This operation is thread safe.
     */
    fn defer_begin(&self) -> bool {
        unsafe { ecs_defer_begin(self.world) }
    }

    /** End block of operations to defer.
     * See defer_begin.
     *
     * This operation is thread safe.
     */
    fn defer_end(&self) -> bool {
        unsafe { ecs_defer_end(self.world) }
    }

    /** Test whether deferring is enabled.
     */
    fn is_deferred(&self) -> bool {
        unsafe { ecs_is_deferred(self.world) }
    }

    /** Test whether the current world object is readonly.
     * This function allows the code to test whether the currently used world
     * object is readonly or whether it allows for writing.
     *
     * @return True if the world or stage is readonly.
     */
    fn is_readonly(&self) -> bool {
        unsafe { ecs_stage_is_readonly(self.world) }
    }

    pub fn find_entity(&self, entity: EntityId) -> Option<Entity> {
        let entity = Entity::new(self.world, entity);
        if entity.is_valid() {
            return Some(entity);
        }
        None
    }

    pub fn lookup(&self, name: &str) -> Option<Entity> {
        let name_c_str = std::ffi::CString::new(name).unwrap();
        let sep = NAME_SEP.as_ptr() as *const i8;

        let entity = unsafe {
            ecs_lookup_path_w_sep(
                self.world,
                0,
                name_c_str.as_ptr() as *const i8,
                sep,
                sep,
                true,
            )
        };

        if entity > 0 {
            return Some(Entity::new(self.world, entity));
        }

        None
    }

    pub fn name(&self, entity: Entity) -> &str {
        let name_str = unsafe { ecs_get_name(self.world, entity.raw()) };
        unsafe { flecs_to_rust_str(name_str) }
    }

    /// Set a singleton component
    pub fn set_singleton<T: Component>(&mut self, value: T) {
        // insert the singleton type automatically if necessary
        if self.id::<T>().is_none() {
            self.component::<T>();
        }

        let comp_id = self.id::<T>().unwrap();
        let entity = comp_id.clone(); // entity = the component for singleton
        self.set(entity, value);
    }

    /// Get a singleton component mutably
    pub fn get_singleton_mut<'a, T: Component>(&'a mut self) -> Option<&'a mut T> {
        // insert the singleton type automatically if necessary
        if self.id::<T>().is_none() {
            self.component::<T>();
        }

        let comp_id = self.id::<T>().unwrap();
        let entity = comp_id.clone(); // entity = the component for singleton

        let dest = unsafe { ecs_get_mut_id(self.world, entity.raw(), comp_id.raw()) };

        if dest.is_null() {
            return None;
        }
        Some(unsafe { (dest as *mut T).as_mut().unwrap() })
    }

    /// Get a singleton component
    pub fn get_singleton<'a, T: Component>(&'a self) -> Option<&'a T> {
        let comp = self.id::<T>().expect("singleton entity does not exist");
        let entity = comp.clone(); // entity = the component for singleton
        self.get_internal::<T>(entity, comp.raw())
    }

    // TODO: should we make this return an option over panicing?
    pub fn get<'a, T: Component>(&'a self, entity: Entity) -> Option<&'a T> {
        let comp_id = WorldInfoCache::get_component_id_for_type::<T>(self.world)
            .expect("Component type not registered!");
        self.get_internal::<T>(entity, comp_id)
    }

    fn get_internal<'a, T: Component>(&'a self, entity: Entity, comp: u64) -> Option<&'a T> {
        let value = unsafe { ecs_get_id(self.world, entity.raw(), comp) };
        if value.is_null() {
            return None;
        }
        Some(unsafe { (value as *const T).as_ref().unwrap() })
    }

    pub fn add<T: Component>(&self, entity: Entity) {
        // flecs_static_assert(is_flecs_constructible<T>::value,
        //     "cannot default construct type: add T::T() or use emplace<T>()");
        let comp_id = WorldInfoCache::get_component_id_for_type::<T>(self.world)
            .expect("Component type not registered!");
        unsafe { ecs_add_id(self.world, entity.raw(), comp_id) };
    }

    pub fn set<T: Component>(&self, entity: Entity, value: T) {
        let comp_id = WorldInfoCache::get_component_id_for_type::<T>(self.world)
            .expect("Component type not registered!");
        let dest = unsafe { ecs_get_mut_id(self.world, entity.raw(), comp_id) };
        let dest = unsafe { (dest as *mut T).as_mut().unwrap() };
        *dest = value;
    }

    pub fn set_component(&self, entity: EntityId, comp: EntityId, data: &[u8]) {
        let info = get_component_info(self.world, comp).expect("Component type not registered!");
        let dest = unsafe {
            let ptr = ecs_get_mut_id(self.world, entity, comp) as *mut u8;
            std::slice::from_raw_parts_mut(ptr, info.size as usize)
        };

        if data.len() == dest.len() {
            dest.copy_from_slice(&data);
        } else {
            // return an error?
            //warn!("set_component: component size mismatch. {} != {}", data.len(), dest.len());
        }
    }

    pub fn read_component(&self, entity: EntityId, comp: EntityId) -> Option<&[u8]> {
        let info = get_component_info(self.world, comp).expect("Component type not registered!");

        let entity_valid = unsafe { ecs_is_valid(self.world, entity) };
        if !entity_valid {
            return None;
        }

        let src = unsafe {
            let ptr = ecs_get_id(self.world, entity, comp) as *const u8;
            if ptr.is_null() {
                return None;
            }
            std::slice::from_raw_parts(ptr, info.size as usize)
        };

        assert!(src.len() == info.size as usize);
        Some(src)
    }

    pub fn write_component<F: FnMut(&mut [u8])>(
        &self,
        entity: EntityId,
        comp: EntityId,
        mut writer: F,
    ) {
        let info = get_component_info(self.world, comp).expect("Component type not registered!");
        let dest = unsafe {
            let ptr = ecs_get_mut_id(self.world, entity, comp) as *mut u8;
            std::slice::from_raw_parts_mut(ptr, info.size as usize)
        };

        writer(dest);
    }

    pub fn id<T: Component>(&self) -> Option<Entity> {
        let type_id = TypeId::of::<T>();

        // see if we already cached it
        if let Some(comp_id) = WorldInfoCache::get_component_id_for_type::<T>(self.world) {
            return Some(Entity::new(self.world, comp_id));
        }
        None
    }

    pub fn component_id<T: Component>(&mut self) -> u64 {
        let comp_id = WorldInfoCache::get_component_id_for_type::<T>(self.world)
            .expect("Component type not registered!");
        comp_id
    }

    pub fn component<T: 'static>(&mut self) -> Entity {
        let comp_id = register_component_typed::<T>(self.world, None);
        Entity::new(self.world, comp_id)
    }

    pub fn component_named<T: 'static>(&mut self, name: &str) -> EntityId {
        register_component_typed::<T>(self.world, Some(name))
    }

    pub fn component_dynamic(&mut self, symbol: &'static str, layout: Layout) -> EntityId {
        register_component_dynamic(self.world, symbol, None, layout)
    }

    pub fn component_dynamic_named(
        &mut self,
        symbol: &'static str,
        name: &'static str,
        layout: Layout,
    ) -> EntityId {
        register_component_dynamic(self.world, symbol, Some(name), layout)
    }

    /** Count entities matching a component id.
     *
     * @param component_id The component id.
     */
    fn count_component_id(&self, component_id: EntityId) -> i32 {
        unsafe { ecs_count_id(self.world, component_id) }
    }

    /** Count entities matching a component by type.
     *
     * @tparam T The component type.
     */
    pub fn count_component<T: 'static>(&self) -> i32 {
        let component_id = register_component_typed::<T>(self.world, None);
        unsafe { ecs_count_id(self.world, component_id) }
    }

    /** Remove all instances of specified component id. */
    fn remove_all_with_component_id(&mut self, component_id: EntityId) {
        unsafe {
            ecs_remove_all(self.world, component_id);
        }
    }

    /** Remove all instances of specified component. */
    pub fn remove_all_with_component<T: 'static>(&mut self) {
        let component_id = register_component_typed::<T>(self.world, None);
        unsafe {
            ecs_remove_all(self.world, component_id);
        }
    }

    /** Check if entity id exists in the world.
     *
     * @see ecs_exists
     */
    fn exists(&self, e: EntityId) -> bool {
        unsafe { ecs_exists(self.world, e) }
    }

    /** Check if entity id exists in the world.
     *
     * @see ecs_is_alive
     */
    fn is_alive(&self, e: EntityId) -> bool {
        unsafe { ecs_is_alive(self.world, e) }
    }

    /** Check if entity id is valid.
     * Invalid entities cannot be used with API functions.
     *
     * @see ecs_is_valid
     */
    fn is_valid(&self, e: EntityId) -> bool {
        unsafe { ecs_is_valid(self.world, e) }
    }

    // Systems

    pub fn system(&self) -> SystemBuilder {
        let sb = SystemBuilder::new(self);
        sb
    }

    // Filters

    pub fn filter<'a, G: ComponentGroup<'a>>(&'a self) -> FilterGroup<'a, G> {
        let filter: FilterGroup<'a, G> = FilterGroup::new(self);
        filter
    }

    pub fn filter_builder(&self) -> FilterBuilder {
        let filter_builder = FilterBuilder::new(self);
        filter_builder
    }

    pub fn query(&self) -> QueryBuilder {
        let builder = QueryBuilder::new(self);
        builder
    }

    // Iterate through all entities matching 1 component
    // TODO: can eliminate this in favor of more general each() once I can fix the
    // single macro issues
    pub fn each1<A: Component>(&self, mut cb: impl FnMut(Entity, &A)) {
        let filter = Filter::new_1::<A>(self.raw());
        filter.each_1(|e: Entity, a: &A| {
            cb(e, a);
        });
    }

    // Rust compiler will not let is use these short forms, perhaps we can solve the errors
    //
    pub fn each<'a, G: ComponentGroup<'a>>(&'a self, cb: impl FnMut(Entity, G::RefTuple)) {
        let filter: FilterGroup<'a, G> = FilterGroup::new(self);
        filter.each(cb);
    }

    pub fn each_mut<'a, G: ComponentGroup<'a>>(&'a self, cb: impl FnMut(Entity, G::MutRefTuple)) {
        let filter: FilterGroup<'a, G> = FilterGroup::new(self);
        filter.each_mut(cb);
    }

    /** Load plecs string.
     * @see ecs_plecs_from_str
     */
    fn plecs_from_str(&mut self, name: &str, plecs_str: &str) -> i32 {
        let name_c_str = std::ffi::CString::new(name).unwrap();
        let plecs_c_str = std::ffi::CString::new(plecs_str).unwrap();
        unsafe {
            ecs_plecs_from_str(
                self.world,
                name_c_str.as_ptr() as *const i8,
                plecs_c_str.as_ptr() as *const i8,
            )
        }
    }

    /** Load plecs from file.
     * @see ecs_plecs_from_file
     */
    fn plecs_from_file(&mut self, filename: &str) -> i32 {
        let filename_c_str = std::ffi::CString::new(filename).unwrap();
        unsafe { ecs_plecs_from_file(self.world, filename_c_str.as_ptr() as *const i8) }
    }

    /** Serialize world to JSON.
     */
    fn to_json(&self) -> String {
        unsafe {
            let json_str = ecs_world_to_json(self.world, std::ptr::null());
            flecs_to_rust_string(json_str)
        }
    }

    /** Deserialize JSON into world.
     */
    fn from_json(&mut self, json: &str) {
        //, flecs::from_json_desc_t *desc = nullptr) {
        let json_c_str = std::ffi::CString::new(json).unwrap();
        let desc = std::ptr::null();
        unsafe {
            let result = ecs_world_from_json(self.world, json_c_str.as_ptr() as *const i8, desc);
        }
    }
}

impl Drop for World {
    fn drop(&mut self) {
        unsafe {
            if self.owned && ecs_stage_is_async(self.world) {
                ecs_async_stage_free(self.world);
            } else if self.owned && !self.world.is_null() {
                ecs_fini(self.world);
            }
        }
    }
}

// Additional Add-ons support
impl World {
    pub fn enable_rest(&self) {
        let rest_comp_id = unsafe { FLECS__EEcsRest as u64 };
        let rest_comp_size = std::mem::size_of::<EcsRest>();

        let rest_data: EcsRest = unsafe { MaybeUninit::zeroed().assume_init() };

        unsafe {
            ecs_set_id(
                self.raw(),
                0,
                rest_comp_id,
                rest_comp_size,
                &rest_data as *const EcsRest as *const ::std::os::raw::c_void,
            )
        };
    }
}

#[cfg(test)]
mod world_tests {
    use super::*;
    struct CompA {
        v: i32,
    }
    struct CompB {
        v: f32,
    }

    fn create_test_world() -> World {
        let mut world = World::new();

        world.component::<CompA>().named("CompA");
        world.component::<CompB>().named("CompB");

        world
            .entity()
            .set(CompA { v: 1234 })
            .set(CompB { v: 123.0 });
        world.entity().set(CompA { v: 2468 }).set(CompB { v: 99.0 });

        world
    }

    #[test]
    fn world_new() {
        let world = create_test_world();
        assert_eq!(world.count_component::<CompA>(), 2);
    }

    #[test]
    fn world_reset() {
        let mut world = create_test_world();
        assert_eq!(world.count_component::<CompA>(), 2);

        world.reset();
        // we must re-register all components!
        world.component::<CompA>();
        world.component::<CompB>();

        assert_eq!(world.count_component::<CompA>(), 0);
    }

    #[test]
    fn world_remove_all() {
        let mut world = create_test_world();
        assert_eq!(world.count_component::<CompA>(), 2);
        world.remove_all_with_component::<CompA>();
        assert_eq!(world.count_component::<CompA>(), 0);
    }

    #[test]
    fn world_load_plecs() {
        let mut world = create_test_world();
        let plecs = r"
			// To see what the result of parsing this file looks like, copy the code and
			// paste it into the editor at https://flecs.dev/explorer
			//
			using flecs.meta
			
			// Create component types, see reflection example
			Struct Position {
				x :- {f32}
				y :- {f32}
			}
			
			Struct Rectangle {
				width :- {f32}
				height :- {f32}
			}
			
			// Plecs files can contain variables that can be referenced later on when 
			// assigning values to components
			const width = 5
			
			// Variables and components can be assigned using expressions. Most arithmetic
			// and conditional operators are supported.
			const height = $width * 2
			
			e {
				- Position{0, -($height / 2)}
				- Rectangle{$width, $height}
			}		
		";

        let result = world.plecs_from_str("some_name", plecs);
        assert_eq!(result, 0);

        // We can lookup the dynamic Component IDs
        let position_component: EntityId = world.lookup("Position").unwrap().into();
        assert_eq!(world.count_component_id(position_component), 1);
    }

    #[test]
    fn world_json() {
        let world = create_test_world();
        let json = world.to_json();
        assert_eq!(json.contains("CompA"), true);

        let mut world2 = World::new();
        world2.component::<CompA>().named("CompA");
        world2.component::<CompB>().named("CompB");

        world2.from_json(&json);
        assert_eq!(world.count_component::<CompA>(), 2);
    }
}
