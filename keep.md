# Trim and Keep Operations

## Overview

The `Trim` operation is a new WereSoCool operation that allows selective extraction of segments from a musical sequence. It works similarly to `ModulateBy` in how it processes a list of operations, but instead of modulating the input, it slices the input into segments and keeps only the specified segment.

## Motivation

Sometimes you want to extract just a portion of a musical phrase or sequence - like taking only the first half, or keeping just the middle section. The `Trim` operation provides a flexible way to do this by allowing you to define cutting points using any WereSoCool operations.

## Syntax

```socool
input | Trim [op1, op2, Keep, op3, ...]
```

Where:
- `op1, op2, ...` are any WereSoCool operations that define segment lengths/cuts
- `Keep` is a special marker indicating which segment to keep
- The position of `Keep` determines which segment is selected

## How It Works

1. **Build the Slicer**: Like `ModulateBy`, `Trim` takes the list of operations and builds a sequence that defines how to slice the input
2. **Find Keep Position**: Locate where `Keep` appears in the operations list 
3. **Calculate Segments**: Use the slicer sequence to determine cut points and segment boundaries
4. **Extract Segment**: Keep only the segment at the position corresponding to `Keep`
5. **Return Result**: The output has the length of the selected segment

## Examples

### Basic Example
```socool
thing1 | Trim [Keep, Fm 0]
```
- Creates 2 segments: first half (Keep) and second half (discarded)
- Returns only the first half of `thing1`

### Complex Example  
```socool
thing1 | Trim [Lm 2, thing1 | Lm 1/2, Keep, Lm 1]
```
- Segment 1: Length ratio 2
- Segment 2: Length from `thing1 | Lm 1/2` 
- Segment 3: **KEEP THIS ONE** (marked with `Keep`)
- Segment 4: Length ratio 1
- Returns only segment 3

### Multiple Keeps
```socool
thing1 | Trim [Lm 1, Keep, Lm 2, Keep, Lm 1]
// Keeps segments 2 and 4, sequencing them
```

When multiple `Keep` markers are used, each marked segment is extracted independently and then they are sequenced using the `Sequence` operation. This allows for creating new compositions by combining different portions of the original sequence in order.

## Implementation Details

### AST Structure
```rust
// In ast.rs Op enum
Trim {
    operations: Vec<Term>,
}

// New marker operation  
Keep,
```

### Core Algorithm (Normalize Implementation)
1. Build slicer sequence using same logic as `ModulateBy`
2. Find index of `Keep` in original operations list
3. Calculate cumulative lengths to determine segment boundaries
4. Extract the operations that fall within the target segment
5. Adjust timing and return the trimmed NormalForm

### Length Calculation
The result length should be the length of the selected segment, not the original input length.

### Parser Grammar
```lalrpop
"Trim" "[" <operations: Operations> "]" => Term::Op(Trim { operations }),
"Keep" => Term::Op(Keep),
```

## Comparison with ModulateBy

| Aspect | ModulateBy | Trim |
|--------|------------|------|
| **Purpose** | Apply modulation pattern to every element | Extract specific segment |
| **Input Processing** | Cross-product (every input × every modulator) | Slice and select |
| **Output Length** | Same as input | Length of selected segment |
| **Result Count** | Multiple modulated versions | Single extracted segment |

## Use Cases

1. **Phrase Extraction**: Get just the melody line from a complex arrangement
2. **Rhythmic Editing**: Extract specific beats or measures  
3. **Dynamic Arrangements**: Use different segments in different contexts
4. **Algorithmic Composition**: Systematically extract portions based on patterns

## Future Enhancements

1. **Multiple Keep Markers**: Allow multiple segments to be selected and overlaid
2. **Named Segments**: `Keep "verse"` for semantic segment selection
3. **Relative Positioning**: `Keep[1]`, `Keep[2]` for numbered selection
4. **Conditional Keeps**: `Keep If <condition>` for dynamic selection

## Related Operations

- `ModulateBy`: Shares the sequence-building logic
- `Sequence`: Defines how segments are concatenated
- `Overlay`: Could be used to combine multiple Trim results
- `Length`: Used within Trim operations to define segment sizes

## Testing Strategy

1. **Basic Cases**: Simple half/quarter splits
2. **Complex Patterns**: Multiple operations with various length ratios
3. **Edge Cases**: Empty segments, Keep at boundaries
4. **Integration**: Combination with other operations
5. **Performance**: Large sequences with many segments

## Implementation Status

- [x] Design completed
- [x] AST enum additions planned  
- [ ] Parser grammar implementation
- [ ] Normalize logic implementation
- [ ] Substitute and get_length_ratio methods
- [ ] Test cases and validation
- [ ] Documentation integration

## Why "Keep" Instead of "Take"?

The name "Take" was already used in WereSoCool's Generator syntax (`GeneratorBase Take N`), so "Keep" was chosen to avoid parser conflicts while maintaining clear semantic meaning.