# Makam model of the language

This is a model of the language in [Makam](https://astampoulis.github.io/makam).
It can be used to drive the renderer and experiment with different choices for
the source language. It is quite slow, so only small examples will work.

## Dependencies

- [Node.js](https://nodejs.org/), version 8.8 and up
- `npm install -g makam@0.7.16`

## Things you can do

- Run the tests:

```
makam --run-tests src/makam/tests
```

- Try things out on the repl:

```
makam src/makam/init
```

You can use the `fastprint_compiler`, e.g.:

```makam
fastprint_compiler {{ Sequence[ AsIs, Tm 0.2 ] | Tm 0.3 }} X ?
```

- Drive the renderer through Makam

```
scripts/render-via-makam songs/bayati.makam
```
