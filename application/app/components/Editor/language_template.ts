export const language_template = `{ f: 311.127, l: 1, g: 1, p: 0 }

thing1 = {
  O[
    (1/1, 2, 1, 1),
    (1/1, 0, 1, -1),
  ]
  | Seq [
    Fm 1, Fm 9/8, Fm 5/4
  ]
}

thing2 = {
  Fm 3/4
  > FitLength thing1
}

main = {
  Overlay [
    thing1,
    thing2
  ]
}
`;

export default language_template;
