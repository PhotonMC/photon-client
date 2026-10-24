# GUI oracle

Screen layout and behaviour are verified by running Java's **own GUI code** headless and recording its draw calls, so guesses from reading the Java source are caught without screenshots.

## How it works

- **Java side.** `tests/behavior/java/v26_2/GuiOracle.java` builds a fake `Minecraft` (through `Unsafe`), `Options`, `Hud`, `ChatListener`, `TextureManager` and sprite/atlas stubs. `GuiGraphicsExtractor` is the real class; every `fill`, sprite (`BlitRenderState`) and text (`GuiTextRenderState`) is read from `GuiRenderState` with its pose applied.
- **Font.** Java's own `font/default.json` providers (bitmap and space) are read from the jar. Glyphs are not drawn, only advances are used, so the layout is Java's layout.
- **Rust side.** `GuiFrame.log` records `DrawOp`s, and `photon-oracle` runs the same scenario through the production Rust screen and emits the same JSON.
- **Comparison.** Java's layer order serves the GPU, so both sides are sorted fill -> sprite -> text, then top to bottom and left to right.

## Writing a GUI case

1. Read the Java screen class line by line. Implementing from someone else's summary is not enough: behaviours such as "the first suggestion is selected" were missed that way.
2. Add an operation in `GuiOracle.java` that drives the real class, and wire it into the dispatch `switch`.
3. Add the scenarios as a case generator under `parity/cases/src/gui/` and list it in that module's `cases()`.
4. Add the matching arm in `crates/photon-oracle/src/gui.rs`. The Rust screen is the production type; the adapter only sets it up and calls `draw`.
5. Run `cargo xtask parity --family gui` (new cases are captured automatically) until it passes. If it differs, fix the Rust side.

Screen patterns:

- Screens: `photon-gui/src/screen/{pause,death,disconnected}.rs`.
- Layout grid: `photon-gui/src/layout.rs`. Widgets: `photon-gui/src/widgets.rs`. Sprites: `photon-render/src/gui/atlas.rs`.
- Game wiring: `crates/photon/src/{controls,gui,play}.rs`.

## Covered operations

| Area | Operations |
| --- | --- |
| HUD | `hud_hotbar`, `hud_status`, `titles`, `boss_bar`, `locator_bar`, `sidebar`, `tab_list`, `name_tags` |
| Chat and commands | `chat`, `chat_draft`, `suggestions`, `command_parse`, `command_suggest` |
| Menus | `title_screen`, `pause_screen`, `death_screen`, `disconnected_screen`, `connect_screen`, `credits_screen`, `confirm_screen`, `dialog_screen`, `game_mode_switcher`, `social_interactions_screen`, `panorama_view` |
| Multiplayer | `join_multiplayer_screen`, `server_form` |
| Options | `widgets`, `options_screen`, `video_settings_screen`, `chat_options_screen`, `mouse_settings_screen`, `controls_screen`, `sound_options_screen`, `skin_customization_screen`, `key_binds_screen`, `key_mappings`, `language_screen`, `lang_text` |
| Containers and items | `container_screen`, `tooltip_lines`, `bundle_tooltip` |
| Text editing | `sign_texts`, `sign_edit_screen`, `book_edit_screen`, `book_sign_screen`, `book_view` |

The scenarios of each operation are in its generator under `parity/cases/src/gui/`.

## Known limits

- **Not verified.** Glyph rendering, items, entities and GPU layering are not verified; only rectangles, sprites and text position and colour are.
- **Selection highlight.** `GUI_TEXT_HIGHLIGHT` is a dedicated pipeline in Java and a plain fill in Rust.
- **Recipe book.** It has no oracle (`RecipeBookComponent` needs much more fake state); it is checked with unit tests written from the Java source. Screens are compared without deferred tooltips.
- **Container screens.**
  - Container screens whose result is computed client-side by the Java menu (anvil cost label, grindstone result) have no filled-result rows.
  - Loom patterns and the cartography map preview have no filled rows, and the smithing armor stand preview is not drawn.
- **Fonts.** Languages with non-Latin names are not listed, because font fallbacks and right-to-left shaping are not ported.
- **Key names.** GLFW gives printable keys their names on the keyboard's layout; a headless run has no window to ask, so the digit keys come out as their language keys. The `hud_debug_overlay` operation adds the US names (`1` to `4`) to the language it runs under; another case that prints a key's name needs the same.
- **Debug overlay.** `hud_debug_overlay` runs the game's own overlay with only the entries that need no running game (version, position, section position); the frame rate, memory and system entries read state the headless scene lacks and are not compared.

## Banner pattern sprites

The recorder answers a sprite of the banner pattern sheet with a texture of its own named for the pattern, so the loom's pattern buttons record which part of the 64 by 64 they show; any other sprite of that lookup fails. The banner drawn in the loom's result slot is a picture-in-picture element that neither side records.
