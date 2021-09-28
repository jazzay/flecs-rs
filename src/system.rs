use super::*;


lazy_static::lazy_static! {
    static ref NAME_SEP: std::ffi::CString = {
		let sep = std::ffi::CString::new("::").unwrap();
		sep
    };
}

pub struct System {
	world: *mut ecs_world_t,
	id: ecs_entity_t,
}

impl System {
	pub(crate) fn new(world: *mut ecs_world_t, id: ecs_entity_t) -> Self {
		System {
			world,
			id
		}
	}

    pub fn interval(&self, interval: f32) {
        unsafe { ecs_set_interval(self.world, self.id, interval) };
    }

    pub fn enable(&self) {
        unsafe { ecs_enable(self.world, self.id, true) };
    }

    pub fn disable(&self) {
        unsafe { ecs_enable(self.world, self.id, false) };
    }

	pub fn run(&self, delta_time: f32) {
		let param: *mut ::std::os::raw::c_void = std::ptr::null_mut();

		unsafe {
			let _last_entity = ecs_run(
				self.world,
				self.id,
				delta_time,
				param,
			);
		}
	}

}

pub struct SystemBuilder {
	world: *mut ecs_world_t,
	desc: ecs_system_desc_t,
	name_temp: String,	// we have to keep it in memory until after build
}

impl SystemBuilder {
	pub(crate) fn new(world: *mut ecs_world_t) -> Self {
		let mut desc = ecs_system_desc_t::default();

		// m_desc.entity.name = name;
		desc.entity.sep = NAME_SEP.as_ptr() as *const i8;
		desc.entity.add[0] = unsafe { EcsOnUpdate };
		// m_desc.query.filter.expr = expr;
		// this->populate_filter_from_pack();

		SystemBuilder {
			world,
			desc,
			name_temp: "".to_owned()
		}
	}

    pub fn name(mut self, name: &str) -> Self {
        self.name_temp = name.to_owned();
		self
    }

    // fn signature(mut self, signature: &str) -> Self {
    //     self.desc.query.filter.expr = signature;
    //     self
    // }

    pub fn interval(mut self, interval: f32) -> Self {
        self.desc.interval = interval;
		self
    }

	/** Associate system with entity */
	pub fn entity(mut self, entity: Entity) -> Self {
		self.desc.self_ = entity.raw();
		self
	}
	
    /** Set system context */
    pub fn ctx(mut self, ctx: *mut ::std::os::raw::c_void) -> Self {
        self.desc.ctx = ctx;
        self
    }	

	// Build APIs, the 2 variants call the internal build()
	fn build(&mut self) -> ecs_entity_t {
		let e: ecs_entity_t;

		let name_c_str = std::ffi::CString::new(self.name_temp.as_str()).unwrap();
		self.desc.entity.name = name_c_str.as_ptr() as *const i8;

        // auto ctx = FLECS_NEW(Invoker)(std::forward<Func>(func));
        // entity_t e, kind = m_desc.entity.add[0];
        // bool is_trigger = kind == flecs::OnAdd || kind == flecs::OnRemove;

        /*if (is_trigger) {
            ecs_trigger_desc_t desc = {};
            ecs_term_t term = m_desc.query.filter.terms[0];
            if (ecs_term_is_initialized(&term)) {
                desc.term = term;
            } else {
                desc.expr = m_desc.query.filter.expr;
            }

            desc.entity.entity = m_desc.entity.entity;
            desc.events[0] = kind;
            desc.callback = Invoker::run;
            desc.self = m_desc.self;
            desc.ctx = m_desc.ctx;
            desc.binding_ctx = ctx;
            desc.binding_ctx_free = reinterpret_cast<
                ecs_ctx_free_t>(_::free_obj<Invoker>);

            e = ecs_trigger_init(m_world, &desc);
        } else*/ {
            let desc = self.desc;
            // desc.callback = Some(Invoker::invoke);
            // desc.self = m_desc.self;
            // desc.query.filter.substitute_default = is_each;
            // desc.binding_ctx = ctx;
            // desc.binding_ctx_free = reinterpret_cast<ecs_ctx_free_t>(_::free_obj<Invoker>);

			e = unsafe { ecs_system_init(self.world, &desc) };
        }

        // if (this->m_desc.query.filter.terms_buffer) {
        //     ecs_os_free(m_desc.query.filter.terms_buffer);
        // }

        e
	}

	// inline system<Components ...> system_builder<Components...>::iter(Func&& func) const {
	// 	using Invoker = typename _::iter_invoker<typename std::decay<Func>::type, Components...>;
	// 	flecs::entity_t system = build<Invoker>(std::forward<Func>(func), false);
	// 	return flecs::system<Components...>(m_world, system);
	// }
	
	// inline system<Components ...> system_builder<Components...>::each(Func&& func) const {
	// 	using Invoker = typename _::each_invoker<typename std::decay<Func>::type, Components...>;
	// 	flecs::entity_t system = build<Invoker>(std::forward<Func>(func), true);
	// 	return flecs::system<Components...>(m_world, system);
	// }

	// temp signature until I get component bundles working
	pub fn iter<F: FnMut(&Iter)>(mut self, func: F) -> System {
		extern "C" fn sys_callback(it: *mut ecs_iter_t) {
			if it.is_null() {
				return;
			}

			let it = unsafe { &*it };

			// let ctx = (*it).ctx;
			// if ctx.is_null() {
			// 	return;
			// }
			// let ctx: *mut Invoker = ctx as *mut Invoker;
			// let invoker: Box<Invoker> = unsafe { Box::from_raw(ctx) };
			// let iter = Iter { it };
			// invoker.target(iter);	

			let sys = EntityRef::new(it.system, it.world);
			println!("sys_callback: {}", sys.name());

			// func(Iter { it });		// can't get this to compile :(
		}
		// let ctx = Box::new(Invoker::new(cb));
		// let ctx = Box::into_raw(ctx) as *mut ::std::os::raw::c_void;
		// self = self.ctx(ctx);
		self.desc.callback = Some(sys_callback);
		let e = Self::build(&mut self);
		System::new(self.world, e)
	}
}

pub struct Iter {
	it: *mut ecs_iter_t
}

/* so far this is not panning out
struct Invoker<F> where F: FnMut(&Iter) {
    pub target: F,
}

impl<F> Invoker<F> where F: FnMut(&Iter) {
    fn new(target: F) -> Self {
        Self { target }
    }
}

extern "C" fn run_callback(it: *mut ecs_iter_t) {
	unsafe {
		if it.is_null() {
			return;
		}
		let ctx = (*it).ctx;
		if ctx.is_null() {
			return;
		}
		let ctx: *mut Invoker = ctx as *mut Invoker;
		let invoker: Box<Invoker> = unsafe { Box::from_raw(ctx) };
		let iter = Iter { it };
		invoker.target(iter);
	}
}
*/

impl Default for ecs_system_desc_t {
    fn default() -> Self {
		let desc: ecs_system_desc_t = unsafe { MaybeUninit::zeroed().assume_init() };
		desc
    }
}