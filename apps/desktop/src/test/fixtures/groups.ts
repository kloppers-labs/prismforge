// Generated from real sessions by `cargo test -p prismforge-desktop --test fixtures`. Do not edit.
import type { GroupsView } from "../../lib/api/bindings/GroupsView";

export const groupsEmpty: GroupsView = {
  "kind": "Demo",
  "groups": [],
  "notices": [],
  "name_limit": 60,
  "group_limit": 100,
  "member_limit": 256
};

export const groupsDemo: GroupsView = {
  "kind": "Demo",
  "groups": [
    {
      "id": "g-61c719836b903612",
      "name": "Desk",
      "members": [
        {
          "target": {
            "device": "demo-graphics-card",
            "zone": null
          },
          "label": "Demo Graphics Card",
          "zone_name": null,
          "state": "Resolved"
        },
        {
          "target": {
            "device": "demo-motherboard",
            "zone": 1
          },
          "label": "Demo Motherboard",
          "zone_name": "Addressable Header 1",
          "state": "Resolved"
        }
      ]
    },
    {
      "id": "g-dbb4af912d1cc46c",
      "name": "Memory",
      "members": [
        {
          "target": {
            "device": "demo-memory-1",
            "zone": null
          },
          "label": "Demo Memory Module",
          "zone_name": null,
          "state": "Resolved"
        },
        {
          "target": {
            "device": "demo-memory-2",
            "zone": null
          },
          "label": "Demo Memory Module",
          "zone_name": null,
          "state": "Resolved"
        }
      ]
    }
  ],
  "notices": [],
  "name_limit": 60,
  "group_limit": 100,
  "member_limit": 256
};

export const groupsRealMissing: GroupsView = {
  "kind": "Real",
  "groups": [
    {
      "id": "g-11e0d895772563cc",
      "name": "Shelf and desk",
      "members": [
        {
          "target": {
            "device": "dev-f529d1aa409f35ce",
            "zone": null
          },
          "label": "Fixture Shelf Strip",
          "zone_name": null,
          "state": "Missing"
        },
        {
          "target": {
            "device": "dev-4947ff71bba71ad6",
            "zone": null
          },
          "label": "Fixture Desk Strip",
          "zone_name": null,
          "state": "Resolved"
        }
      ]
    }
  ],
  "notices": [],
  "name_limit": 60,
  "group_limit": 100,
  "member_limit": 256
};
