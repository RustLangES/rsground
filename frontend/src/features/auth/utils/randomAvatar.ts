export function randomAvatar(): string {
  const avatars = [
    "https://cdn.rsground.rustlang-es.org/avatar-0.webp",
    "https://cdn.rsground.rustlang-es.org/avatar-1.webp",
    "https://cdn.rsground.rustlang-es.org/avatar-2.webp",
    "https://cdn.rsground.rustlang-es.org/avatar-3.webp",
    "https://cdn.rsground.rustlang-es.org/avatar-4.webp",
    "https://cdn.rsground.rustlang-es.org/avatar-5.webp",
    "https://cdn.rsground.rustlang-es.org/avatar-6.webp",
    "https://cdn.rsground.rustlang-es.org/avatar-7.webp",
    "https://cdn.rsground.rustlang-es.org/avatar-8.webp",
    "https://cdn.rsground.rustlang-es.org/avatar-9.webp",
    "https://cdn.rsground.rustlang-es.org/avatar-10.webp",
    "https://cdn.rsground.rustlang-es.org/avatar-11.webp",
    "https://cdn.rsground.rustlang-es.org/avatar-12.webp",
    "https://cdn.rsground.rustlang-es.org/avatar-13.webp",
    "https://cdn.rsground.rustlang-es.org/avatar-14.webp",
    "https://cdn.rsground.rustlang-es.org/avatar-15.webp",
    "https://cdn.rsground.rustlang-es.org/avatar-16.webp",
  ];

  return avatars[Math.random() * avatars.length | 0];
}
