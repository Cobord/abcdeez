# Test Compilation Fixes Needed

## Issues to Fix:
1. `distance` -> `get_distance` 
2. `get_node_index` doesn't exist - need alternative approach
3. `practice_count` field doesn't exist on MemoryStrength
4. `IndexMapping` variant doesn't exist in OperationType
5. Type annotations needed for float exp()
6. `forward` field doesn't exist in KJump
7. Can't compare PosteriorDistribution with >= and <=

## Summary:
Our new test files introduce comprehensive testing but need API adjustments to match the actual implementation.