# handlers/task_simple.rs - Simple Task Generation Interface

## Requirements and Dataflow
- Provides simplified task generation endpoints for basic learning interactions
- Integrates with core learning algorithms for adaptive task creation
- Supports difficulty level queries and hint generation
- Offers lightweight task interface for mobile and embedded applications
- Maintains compatibility with existing simple task workflows

## High-level Purpose and Responsibilities
- **Simple Task Generation**: Lightweight task creation without full session complexity
- **Difficulty Management**: Dynamic difficulty assessment and adjustment
- **Hint System**: Context-aware hint generation for learner support
- **Mobile Compatibility**: Simplified endpoints optimized for mobile applications
- **Legacy Support**: Backward compatibility with existing simple task interfaces
- **Core Integration**: Bridge between simple endpoints and sophisticated core algorithms

## Key Abstractions and Interfaces
- Simple task generation with minimal configuration requirements
- Difficulty level queries with adaptive adjustment recommendations
- Hint generation system with contextual learning support
- Lightweight response format optimized for mobile and embedded use
- Integration layer connecting simple interface to core learning algorithms

## Data Transformations and Flow
1. **Task Generation**: Request parameters → core algorithm invocation → task creation → response formatting
2. **Difficulty Assessment**: Learning context → difficulty calculation → level recommendation → client response
3. **Hint Generation**: Task context → hint algorithm → contextual clues → formatted hints
4. **Core Integration**: Simple request → core library translation → algorithm execution → simplified response
5. **Mobile Optimization**: Complex data → response simplification → bandwidth optimization → client delivery

## Dependencies and Interactions
- **Core Learning Library**: Advanced task generation and adaptive algorithms
- **Session System**: Optional integration with full session management
- **Learner Profiles**: Performance history for difficulty adaptation
- **Hint System**: Contextual help generation and learning support
- **Mobile Clients**: Optimized endpoints for mobile and embedded applications
- **Backward Compatibility**: Support for existing simple task workflows

## Architectural Patterns
- **Simplified Interface**: Lightweight endpoints abstracting complex core functionality
- **Core Integration**: Clean translation layer between simple API and sophisticated algorithms
- **Mobile Optimization**: Response format and size optimization for mobile clients
- **Adaptive Algorithms**: Dynamic difficulty and hint generation based on learner context
- **Backward Compatibility**: Maintenance of existing API contracts and workflows
- **Progressive Enhancement**: Simple base functionality with optional advanced features