import { describe, it, expect } from "vitest";
import { greet } from "../src/index";

describe("greet", () => {
  it("should return greeting", () => {
    expect(greet()).toBe("Hello, World!");
  });
});
