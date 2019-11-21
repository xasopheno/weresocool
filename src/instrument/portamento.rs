(p.f = op.c.f[-1] * o.offset[-1])
c.f = ((op.c.f - op.c.p) * s+index/ts + op.c.p) * offset.f

if let p_delta = p.silent || c.silent || c.f == c.p || p_index > p_length {
  1.
} else {
  (c.f - p.f) / (p_length)
}

/// frequency is a function of past op and current op
fn calculate_portamento_delta() {
    
}
