use crate::*;
use crate::cache::WorldInfoCache;

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/// ComponentGroup prototype (WIP)
/// 
/// This will allow us to Rustify all the Filter, Query, and System APIs by support Rust tuples
/// 
/// Insipred by: https://github.com/HindrikStegenga/Shard

use private::SealedComponentGroup;

#[macro_export]
macro_rules! expr {
    ($x:expr) => {
        $x
    };
}

#[macro_export]
macro_rules! tuple_index {
    ($tuple:expr, $idx:tt) => {
        expr!($tuple.$idx)
    };
}

/// Represents a group of components. Used for specifying which component types should be matched in query's.
pub trait ComponentGroup<'c>: /*private::SealedComponentGroup +*/ Sized + 'static {
    type RefTuple: 'c;
    type MutRefTuple: 'c;

    /// Populates an ecs_filter_desc with the component type ids
    unsafe fn fill_descriptor(world: *mut ecs_world_t, desc: &mut ecs_filter_desc_t);

    /// Assembles a component tuple from an active iterator
    unsafe fn iter_as_ref_tuple(it: &ecs_iter_t, i: isize) -> Self::RefTuple;

    /// Assembles a mutable component tuple from an active iterator
    unsafe fn iter_as_mut_tuple(it: &ecs_iter_t, i: isize) -> Self::MutRefTuple;
}


impl<'c, T: Component + SealedComponentGroup> ComponentGroup<'c> for T {
    type RefTuple = &'c T;
    type MutRefTuple = &'c mut T;

    unsafe fn fill_descriptor(world: *mut ecs_world_t, desc: &mut ecs_filter_desc_t) {
        desc.terms[0].id = WorldInfoCache::get_component_id_for_type::<T>(world).expect("Component type not registered!");
    }

    unsafe fn iter_as_ref_tuple(it: &ecs_iter_t, i: isize) -> Self::RefTuple {
        let v = ecs_field::<T>(it, 1).offset(i as isize).as_ref().unwrap();
        &*(v)
    }

    unsafe fn iter_as_mut_tuple(it: &ecs_iter_t, i: isize) -> Self::MutRefTuple {
        let v = ecs_field::<T>(it, 1).offset(i as isize).as_mut().unwrap();
        &mut *(v)
    }
}

macro_rules! impl_component_tuple {
    ($len:expr, $(($elem:ident, $elem_idx:tt)), *) => {
        impl<'s, $($elem),*> ComponentGroup<'s> for ($($elem), *)
        where $( $elem : Component /*+ SealedComponentGroup*/ ),*/*, Self : SealedComponentGroup*/
        {
            type RefTuple = ($(&'s $elem),*);
            type MutRefTuple = ($(&'s mut $elem),*);

            unsafe fn fill_descriptor(world: *mut ecs_world_t, desc: &mut ecs_filter_desc_t) {
                $(
                    desc.terms[$elem_idx].id = WorldInfoCache::get_component_id_for_type::<$elem>(world).expect("Component type not registered!");
                )*
            }
        
            // We should be able to split this into 2 phase to gain performance (ecs_term, and the offset maths on raw ptr)
            unsafe fn iter_as_ref_tuple(it: &ecs_iter_t, i: isize) -> Self::RefTuple {
                ($(
                    &*((ecs_field::<$elem>(it, $elem_idx + 1)) as *mut $elem).offset(i as isize).as_ref().unwrap(),
                )*)
            }

            unsafe fn iter_as_mut_tuple(it: &ecs_iter_t, i: isize) -> Self::MutRefTuple {
                ($(
                    &mut *((ecs_field::<$elem>(it, $elem_idx + 1)) as *mut $elem).offset(i as isize).as_mut().unwrap(),
                )*)
            }
        }
    }
}

impl_component_tuple!(8, (T1, 0), (T2, 1), (T3, 2), (T4, 3), (T5, 4), (T6, 5), (T7, 6), (T8, 7));
impl_component_tuple!(7, (T1, 0), (T2, 1), (T3, 2), (T4, 3), (T5, 4), (T6, 5), (T7, 6));
impl_component_tuple!(6, (T1, 0), (T2, 1), (T3, 2), (T4, 3), (T5, 4), (T6, 5));
impl_component_tuple!(5, (T1, 0), (T2, 1), (T3, 2), (T4, 3), (T5, 4));
impl_component_tuple!(4, (T1, 0), (T2, 1), (T3, 2), (T4, 3));
impl_component_tuple!(3, (T1, 0), (T2, 1), (T3, 2));
impl_component_tuple!(2, (T1, 0), (T2, 1));
// impl_component_tuple!(1, (T1, 0));       // macro errors result

mod private {
//     use crate::Component;

    pub trait SealedComponentGroup {}

//     impl<'s, T> SealedComponentGroup for T where T: Component {}

//     macro_rules! impl_sealed_component_tuples {
//         ($($ty:ident)*) => {}; //base case
//         ($head:ident, $($tail:ident),*) => {
//             impl<$($tail),*, $head> SealedComponentGroup for impl_sealed_component_tuples!([$($tail)*] $head)
//             where $head : Component, $( $tail : Component ),* {}

//             impl_sealed_component_tuples!($($tail),*);
//         };
//         ([] $($ty:ident)*) => {
//             ($($ty), *)
//         };
//         ([$first:ident $($tail:ident)*] $($ty:ident)*) => {
//             impl_sealed_component_tuples!([$($tail)*] $first $($ty)*)
//         };
//     }

//     impl_sealed_component_tuples!(
//         T16, T15, T14, T13, T12, T11, T10, T9, T8, T7, T6, T5, T4, T3, T2, T1
//     );
}