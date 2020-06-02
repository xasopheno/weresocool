export const language_template = `{ f: 285, l: 1, g: 1, p: 0 }

overtones = {
  O[
    (1/1, 2, 1, 1),
    (1/1, 0, 1, -1),
  ]
}

thing1 = {
  Seq [
    Fm 1, Fm 9/8, Fm 5/4
  ]
}

main = {
  overtones
  | thing1
}
`;

export default language_template;
