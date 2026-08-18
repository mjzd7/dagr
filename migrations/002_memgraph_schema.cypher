// ====================================================================
// DAGR 3D Knowledge Graph: Memgraph MAGE Schema & APOC Indexes
// ====================================================================

// Unique Constraints
CREATE CONSTRAINT ON (s:Symbol) ASSERT s.id IS UNIQUE;
CREATE CONSTRAINT ON (f:File) ASSERT f.id IS UNIQUE;
CREATE CONSTRAINT ON (m:Module) ASSERT m.id IS UNIQUE;
CREATE CONSTRAINT ON (t:DbTable) ASSERT t.id IS UNIQUE;

// Relationship Constraints
// (:Symbol)-[:CALLS {call_count: Int, is_async: Boolean}]->(:Symbol)
// (:Symbol)-[:DECLARED_IN]->(:File)
// (:File)-[:IMPORTS {alias: String}]->(:File)
// (:Symbol)-[:MUTATES_SCHEMA {operation: String}]->(:DbTable)

// Sub-Graph Minimal Extraction Query for 2-Hop Blast Radius Traversal
// MATCH (seed:Symbol {id: $seed_symbol_id})
// CALL apoc.path.subgraphAll(seed, {
//     maxLevel: 2,
//     relationshipFilter: "CALLS>|MUTATES_SCHEMA>|IMPORTS>"
// })
// YIELD nodes, relationships
// RETURN nodes, relationships;
