export function greet(): string {
  return "Hello, World!";
}

if (import.meta.main) {
  console.log(greet());
}
