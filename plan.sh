### frequency

(p.f = op.c.f[-1] * o.offset[-1])
c.f = ((op.c.f - op.c.p) * s+index/ts + op.c.p) * offset.f

if let p_delta = p.silent || c.silent || c.f == c.p || p_index > p_length {
  1.
} else {
  (c.f - p.f) / (p_length)
}

tests:

p.silent: 1. 
c.silent: 1. 
c.f == p.f: 1.
p_index > p_length: 1.

p_index == 0
p.f = 10
c.f = 20


### gain

tests:
if 
  op.p.silent && op.c.silent ||  
  p.silent && c.silent {
  0.0
} 
else if 
  index > 0 &&
  !op.c.silent && 
  c.p.silent &&
  !c.f.silent {

  attack = 1024
}
else if 
  index > &&
  !op.c.silent && 
  !c.p.silent &&
  c.f.silent {
  decay length = 1024
}


(p.g = loudness(c.f[-1]) * op.c.g[-1] * offset.g[-1])
c.g = loudness(c.f) * op.c.g * offset.g

