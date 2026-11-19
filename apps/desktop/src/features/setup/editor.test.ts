import { describe, expect, test } from "vitest";
import type { Geometry } from "../../lib/api/bindings/Geometry";
import type { Setup } from "../../lib/api/bindings/Setup";
import { demoConnected, openRgbSetupDevices } from "../../test/fixtures/sessions";
import { demoTower, demoTowerParts, demoTowerShared, ledSamples } from "../../test/fixtures/setupModel";
import { problems } from "./bounds";
import {
  CANVAS,
  addComponent,
  flip,
  moveBy,
  moveInChain,
  placementOf,
  rename,
  rotateBy,
  setGeometry,
  setRotation,
  unwire,
  VIEWS,
  wire,
  withViews,
} from "./edits";
import { ledCount, viewLeds, withLeds, reversed } from "./geometry";
import { History } from "./history";
import { chainOf, parts, sharedWith } from "./wiring";

const PC = VIEWS.pc;
const tower = (): Setup => structuredClone(demoTower);
const at = (setup: Setup, id: string) => placementOf(setup, PC, id)!;

describe("the shapes and wiring match the Rust model", () => {
  test("each_shape_places_its_leds_where_the_rust_model_does", () => {
    for (const sample of ledSamples) {
      const leds = viewLeds(sample.geometry, sample.placement);
      expect(leds.length, JSON.stringify(sample.geometry)).toBe(sample.leds.length);
      leds.forEach((led, index) => {
        expect(led.x).toBeCloseTo(sample.leds[index]!.x, 6);
        expect(led.y).toBeCloseTo(sample.leds[index]!.y, 6);
      });
    }
  });

  test("parts_and_mirrors_match_the_rust_model", () => {
    const setup = tower();
    expect(setup.outputs.flatMap((output) => parts(setup, output))).toEqual(demoTowerParts);
    for (const { component, shared_with } of demoTowerShared) {
      expect(sharedWith(setup, component)).toEqual(shared_with);
    }
  });
});

describe("editing a component", () => {
  test("a_move_keeps_the_component_inside_the_view", () => {
    const moved = moveBy(tower(), PC, ["c-front-fan-1"], -5000, 20);

    expect(at(moved, "c-front-fan-1")).toMatchObject({ x: 0, y: 150 });
    expect(at(moved, "c-front-fan-2")).toEqual(at(tower(), "c-front-fan-2"));
  });

  test("moving_several_components_moves_them_together_and_keeps_their_spacing", () => {
    const moved = moveBy(tower(), PC, ["c-front-fan-1", "c-rear-fan"], 400, -10);

    expect(at(moved, "c-front-fan-1")).toMatchObject({ x: 250, y: 120 });
    expect(at(moved, "c-rear-fan")).toMatchObject({ x: 1000, y: 120 });
    expect(at(moved, "c-rear-fan").x - at(moved, "c-front-fan-1").x).toBe(750);
  });

  test("rotation_turns_in_fifteen_degree_steps_and_wraps", () => {
    const once = rotateBy(tower(), PC, ["c-graphics"], 1);
    const back = rotateBy(tower(), PC, ["c-graphics"], -1);
    const typed = setRotation(tower(), PC, "c-graphics", 365);

    expect(at(once, "c-graphics").rotation).toBe(15);
    expect(at(back, "c-graphics").rotation).toBe(345);
    expect(at(typed, "c-graphics").rotation).toBe(5);
    expect(at(rotateBy(tower(), PC, ["c-memory-1"], 24), "c-memory-1").rotation).toBe(90);
  });

  test("mirror_flips_the_picture_and_leaves_the_led_order", () => {
    const flipped = flip(tower(), PC, ["c-graphics"]);

    expect(at(flipped, "c-graphics").flipped).toBe(true);
    expect(flipped.components).toEqual(tower().components);
    expect(at(flip(flipped, PC, ["c-graphics"]), "c-graphics").flipped).toBe(false);
  });

  test("the_led_count_first_led_and_direction_change_only_that_component", () => {
    const ring = tower().components.find((component) => component.id === "c-rear-fan")!.geometry;
    const strip = tower().components.find((component) => component.id === "c-graphics")!.geometry;

    const more = setGeometry(tower(), "c-rear-fan", withLeds(ring, 12));
    const back = setGeometry(tower(), "c-graphics", reversed(strip));

    expect(ledCount(more.components.find((component) => component.id === "c-rear-fan")!.geometry)).toBe(12);
    expect(more.components.filter((component) => component.id !== "c-rear-fan")).toEqual(tower().components.filter((component) => component.id !== "c-rear-fan"));
    expect(back.components.find((component) => component.id === "c-graphics")!.geometry).toEqual({ Strip: { leds: 12, first: "AtEnd" } });
    expect(reversed({ Ring: { leds: 8, first_angle: 90, turn: "Clockwise" } })).toEqual({ Ring: { leds: 8, first_angle: 90, turn: "CounterClockwise" } });
  });

  test("a_rename_trims_nothing_until_save_and_keeps_unknown_fields", () => {
    const setup = tower() as Setup & { future?: string };
    setup.future = "kept";
    (setup.components[0] as unknown as Record<string, unknown>).note = "kept too";

    const renamed = rename(setup, "c-front-fan-1", "Top intake") as Setup & { future?: string };

    expect(renamed.components[0]!.name).toBe("Top intake");
    expect(renamed.future).toBe("kept");
    expect((renamed.components[0] as unknown as Record<string, unknown>).note).toBe("kept too");
  });

  test("a_new_setup_has_a_pc_view_and_a_desk_view", () => {
    const setup = withViews({ components: [], outputs: [], views: [] });

    expect(setup.views.map((view) => [view.id, view.name])).toEqual([
      ["v-pc", "PC"],
      ["v-desk", "Desk"],
    ]);
    expect(withViews(tower()).views.map((view) => view.id)).toEqual(["v-pc", "v-desk"]);
  });
});

describe("LED counts against the backend bounds", () => {
  const ring = (leds: number): Geometry => ({ Ring: { leds, first_angle: 0, turn: "Clockwise" } });

  test("a_component_above_the_zone_maximum_gets_the_reason_with_the_bounds", () => {
    const setup = setGeometry(tower(), "c-cooler-left", { Strip: { leds: 9, first: "AtStart" } });

    const found = problems(setup, demoConnected);

    expect(found).toEqual([
      {
        component: null,
        output: "o-cooler",
        text: "The parts on Addressable Header 2 need 17 LEDs, and it has 16. Use fewer LEDs.",
      },
    ]);
  });

  test("a_fan_above_a_resizable_headers_maximum_names_the_maximum", () => {
    const setup = setGeometry(tower(), "c-front-fan-1", ring(121));

    const found = problems(setup, demoConnected);

    expect(found).toEqual([
      { component: "c-front-fan-1", output: "o-fan-hub", text: "Front fan 1 has 121 LEDs. Addressable Header 1 takes at most 120." },
      {
        component: null,
        output: "o-fan-hub",
        text: "The parts on Addressable Header 1 need 121 LEDs, and it has 30. Use fewer LEDs, or resize the zone in OpenRGB first; it takes up to 120.",
      },
    ]);
  });

  test("an_output_of_a_device_the_session_does_not_report_is_not_checked", () => {
    expect(problems(tower(), openRgbSetupDevices)).toEqual([]);
  });

  test("a_fitting_setup_has_no_problem", () => {
    expect(problems(tower(), demoConnected)).toEqual([]);
  });
});

describe("mirrors and daisy chains", () => {
  const target = { device: "demo-motherboard", zone: 1, label: "Demo Motherboard", zoneName: "Addressable Header 1" };
  const added = () =>
    addComponent(tower(), PC, { id: "c-extra", name: "Extra fan", geometry: { Ring: { leds: 8, first_angle: 0, turn: "Clockwise" } } }, { x: 300, y: 600 });

  test("wiring_as_a_mirror_adds_a_branch_that_shows_the_same_colours", () => {
    const setup = wire(added(), "c-extra", target, "mirror", "o-unused");

    const hub = setup.outputs.find((output) => output.id === "o-fan-hub")!;
    expect(hub.branches).toHaveLength(5);
    expect(sharedWith(setup, "c-extra")).toEqual(["c-front-fan-1", "c-front-fan-2", "c-front-fan-3", "c-rear-fan"]);
  });

  test("wiring_as_a_chain_appends_the_next_part_of_the_zone", () => {
    const setup = wire(added(), "c-extra", { ...target, zone: 2, zoneName: "Addressable Header 2" }, "chain", "o-unused");

    expect(chainOf(setup, "c-extra")).toMatchObject({ output: "o-cooler", branch: 0, index: 2, length: 3 });
    expect(parts(setup, setup.outputs.find((output) => output.id === "o-cooler")!).at(-1)).toEqual({ component: "c-extra", branch: 0, start: 16, end: 24 });
  });

  test("wiring_to_an_output_not_in_the_setup_makes_a_new_output_of_its_own", () => {
    const setup = wire(added(), "c-extra", { device: "demo-spare", zone: null, label: "Spare strip", zoneName: null }, "own", "o-light");

    expect(setup.outputs.at(-1)).toEqual({
      id: "o-light",
      device: "demo-spare",
      zone: null,
      label: "Spare strip",
      zone_name: null,
      first_led: 0,
      branches: [["c-extra"]],
    });
  });

  test("unwiring_the_last_part_drops_the_output_and_leaves_the_component_placed", () => {
    const once = unwire(tower(), "c-graphics");

    expect(once.outputs.some((output) => output.id === "o-graphics")).toBe(false);
    expect(at(once, "c-graphics")).toEqual(at(tower(), "c-graphics"));
  });

  test("a_part_moves_later_in_its_chain", () => {
    const setup = moveInChain(tower(), "c-cooler-left", 1);

    expect(setup.outputs.find((output) => output.id === "o-cooler")!.branches).toEqual([["c-cooler-right", "c-cooler-left"]]);
  });
});

describe("undo and redo", () => {
  test("undo_and_redo_restore_the_setup_and_the_selection", () => {
    const history = new History<{ setup: Setup; selection: string[] }>();
    const first = { setup: tower(), selection: ["c-graphics"] };
    const second = { setup: rotateBy(first.setup, PC, ["c-graphics"], 1), selection: ["c-graphics", "c-rear-fan"] };
    history.record(first, "Rotate Graphics card");

    const undone = history.undo(second);
    const redone = history.redo(undone!.state);

    expect(undone).toEqual({ state: first, label: "Rotate Graphics card" });
    expect(redone).toEqual({ state: second, label: "Rotate Graphics card" });
    expect(history.undoLabel).toBe("Rotate Graphics card");
    expect(history.redoLabel).toBeNull();
  });

  test("a_new_edit_after_an_undo_drops_the_redo", () => {
    const history = new History<number>();
    history.record(1, "One");
    history.record(2, "Two");
    history.undo(3);

    history.record(2, "Other");

    expect(history.redoLabel).toBeNull();
    expect(history.undoLabel).toBe("Other");
  });

  test("the_history_keeps_a_bounded_number_of_steps", () => {
    const history = new History<number>(3);
    [1, 2, 3, 4, 5].forEach((value) => history.record(value, String(value)));

    expect(history.undo(6)?.state).toBe(5);
    expect(history.undo(5)?.state).toBe(4);
    expect(history.undo(4)?.state).toBe(3);
    expect(history.undo(3)).toBeNull();
  });
});

test("the_view_extent_is_fixed_so_numbers_mean_the_same_on_every_screen", () => {
  expect(CANVAS).toEqual({ width: 1000, height: 700 });
});
