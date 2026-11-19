import { expect, test } from "vitest";
import { contrast } from "./contrast";
import tokens from "./tokens.css?raw";

function token(name: string): string {
  const found = new RegExp(`--${name}:\\s*(#[0-9a-f]{6})`, "i").exec(tokens);
  if (!found?.[1]) {
    throw new Error(`no colour token --${name}`);
  }
  return found[1];
}

test("an_unlit_led_has_at_least_3_to_1_contrast_on_the_drawing_and_on_a_shape", () => {
  expect(contrast(token("led-off"), token("surface-stage"))).toBeGreaterThanOrEqual(3);
  expect(contrast(token("led-off"), token("shape-fill"))).toBeGreaterThanOrEqual(3);
});
