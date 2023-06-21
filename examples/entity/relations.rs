use flecs::*;

// TODO
// - Extend the various add_relation overloads to support <T>, id as well
// - figure out relation iteration overloads

// Type used for Eats relation
struct Eats {}

fn main() {
	println!("Entity Relations starting...");

	let mut world = World::new();

	// We have to manually register all components
	let eats_id = world.component_named::<Eats>("Eats");

	// Entity used for Grows relation
	let grows = world.entity().named("Grows");

	// Relation objects
	let apples = world.entity().named("Apples");
	let pears = world.entity().named("Pears");

	// Create an entity with 3 relations. Relations are like regular components,
	// but combine two types/identifiers into an (relation, object) pair.
	let bob = world
		.entity()
		.named("Bob")
		// Pairs can be constructed from a type and entity
		// .add_relation::<Eats>(apples)
		.add_relation_ids(eats_id, apples)
		// .add_relation::<Eats>(pears)
		// Pairs can also be constructed from two entity ids
		.add_relation_ids(grows, pears);

	// Has can be used with relations as well
	// println!("Bob eats apples? {}", bob.has_relation::<Eats>(apples));
	println!("Bob eats apples? {}", bob.has_relation(eats_id, apples));

	// Wildcards can be used to match relations
	println!("Bob grows food? {}", bob.has_relation_wildcard(grows));

	// // Print the type of the entity. Should output:
	// //   (Identifier,Name),(Eats,Apples),(Eats,Pears),(Grows,Pears)
	println!("Bob's type: [ {} ]", bob.type_info().to_str());

	// // Relations can be iterated for an entity. This iterates (Eats, *):
	// bob.each<Eats>([](flecs::entity obj) {
	//     std::cout << "Bob eats " << obj.name() << "\n";
	// });

	// // Iterate by explicitly providing the pair. This iterates (*, Pears):
	// bob.each(flecs::Wildcard, pears, [](flecs::id id) {
	//     std::cout << "Bob " << id.relation().name() << " pears\n";
	// });
}

#[cfg(test)]
mod tests {
	#[test]
	fn flecs_entity_relations() {
		super::main();
	}
}
