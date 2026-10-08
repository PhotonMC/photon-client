"""The `gui` family: original synthetic inputs."""

import itertools

from cases.gui.books import gui_book_edit_screen_cases, gui_book_sign_screen_cases, gui_book_view_cases
from cases.gui.chat import gui_cases, gui_chat_draft_cases, gui_suggestion_cases
from cases.gui.commands import command_tree_hex, gui_command_parse_cases, gui_command_suggest_cases
from cases.gui.hud import _waypoint_bytes, gui_boss_bar_cases, gui_hud_hotbar_cases, gui_hud_status_cases, gui_locator_bar_cases, gui_name_tag_cases, gui_sidebar_cases, gui_tab_list_cases, gui_title_cases
from cases.gui.keybinds import gui_key_binds_cases
from cases.gui.language import gui_language_cases, gui_settings_key_cases, gui_settings_key_generators, gui_settings_key_press_cases, latin_language_codes
from cases.gui.screens import gui_confirm_screen_cases, gui_container_screen_cases, gui_death_screen_cases, gui_disconnected_screen_cases, gui_pause_screen_cases, gui_widgets_cases, trader, trades
from cases.gui.signs import gui_sign_edit_screen_cases, gui_sign_text_cases
from cases.gui.title import gui_connect_screen_cases, gui_join_multiplayer_screen_cases, gui_server_form_cases, gui_server_icon_cases, gui_server_list_ping_cases, gui_title_screen_cases
from cases.gui.settings import gui_chat_options_cases, gui_key_mappings_cases, gui_mouse_cases, gui_options_screen_cases, gui_sound_options_cases, gui_video_settings_cases


def gui_generators():
    return itertools.chain(
        gui_cases(), gui_suggestion_cases(), gui_title_cases(), gui_boss_bar_cases(),
        gui_sidebar_cases(), gui_command_parse_cases(), gui_command_suggest_cases(),
        gui_pause_screen_cases(), gui_death_screen_cases(), gui_disconnected_screen_cases(),
        gui_tab_list_cases(), gui_container_screen_cases(),
        gui_confirm_screen_cases(), gui_widgets_cases(),
        gui_options_screen_cases(), gui_video_settings_cases(), gui_chat_options_cases(),
        gui_mouse_cases(), gui_sound_options_cases(), gui_hud_hotbar_cases(), gui_hud_status_cases(), gui_language_cases(), gui_locator_bar_cases(),
        gui_settings_key_generators(), gui_chat_draft_cases(), gui_name_tag_cases(), gui_title_screen_cases(), gui_join_multiplayer_screen_cases(), gui_server_form_cases(), gui_connect_screen_cases(), gui_server_list_ping_cases(), gui_server_icon_cases(), gui_key_mappings_cases(), gui_key_binds_cases(), gui_sign_text_cases(), gui_sign_edit_screen_cases(), gui_book_edit_screen_cases(), gui_book_sign_screen_cases(), gui_book_view_cases())
