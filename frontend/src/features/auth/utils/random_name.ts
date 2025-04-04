export function generateRandomName(): string {
  const firstName = firstNames[Math.random() * firstNames.length | 0]
  const lastName = lastNames[Math.random() * lastNames.length | 0]
  const number = Math.random() * 999 | 0;

  return firstName + lastName + number
}

// You can contribute with fun names.
// Please keep the alphanumeric sort

/** adjectives */
const firstNames = [
  "Oxidized",
  "Rustic",
];

/** subjects */
const lastNames = [
  "Learner",
  "User",
];
