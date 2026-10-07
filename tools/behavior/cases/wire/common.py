"""Helpers of the `wire` family inputs."""

from cases.helpers import *


def varint_hex(value):
    value &= 0xFFFFFFFF
    encoded = []
    while value & ~0x7F:
        encoded.append((value & 0x7F) | 0x80)
        value >>= 7
    encoded.append(value)
    return bytes(encoded).hex()


def utf16_hex(value):
    return value.encode("utf-16-be", "surrogatepass").hex()


def intention_hex(protocol_version, host_bytes, port, intention_id):
    return (varint_hex(protocol_version) + varint_hex(len(host_bytes)) + host_bytes.hex()
            + (port & 0xFFFF).to_bytes(2, "big").hex() + varint_hex(intention_id))


def hello_hex(name_bytes, most_hex, least_hex):
    return (varint_hex(len(name_bytes)) + name_bytes.hex()
            + most_hex + least_hex)


def profile_property(name, value, signature=None):
    return {"name_utf16_hex": utf16_hex(name), "value_utf16_hex": utf16_hex(value),
            "signature_utf16_hex": None if signature is None else utf16_hex(signature)}


def property_hex(name_bytes, value_bytes, signature_bytes=None):
    result = varint_hex(len(name_bytes)) + name_bytes.hex()
    result += varint_hex(len(value_bytes)) + value_bytes.hex()
    if signature_bytes is None:
        return result + "00"
    return result + "01" + varint_hex(len(signature_bytes)) + signature_bytes.hex()


def finished_hex(name_bytes, properties, profile=("0123456789abcdef", "fedcba9876543210"),
                 session=("0011223344556677", "8899aabbccddeeff")):
    return (profile[0] + profile[1] + varint_hex(len(name_bytes)) + name_bytes.hex()
            + varint_hex(len(properties)) + "".join(properties)
            + session[0] + session[1])


def code_of_conduct_frame(value):
    encoded = value.encode("utf-8")
    return varint_hex(len(encoded)) + encoded.hex()


def client_information_hex(language_bytes, view_distance, chat_visibility, chat_colors,
                           model_customisation, main_hand, text_filtering, allows_listing,
                           particle_status):
    return (varint_hex(len(language_bytes)) + language_bytes.hex()
            + (view_distance & 0xff).to_bytes(1, "big").hex()
            + varint_hex(chat_visibility) + ("01" if chat_colors else "00")
            + (model_customisation & 0xff).to_bytes(1, "big").hex()
            + varint_hex(main_hand) + ("01" if text_filtering else "00")
            + ("01" if allows_listing else "00") + varint_hex(particle_status))


def ping_hex(value):
    return int(value).to_bytes(8, "big", signed=True).hex()


def common_fixed_hex(value, width):
    return int(value).to_bytes(width, "big", signed=True).hex()


def feature_string_frame(value):
    encoded = value.encode("utf-8")
    return varint_hex(len(encoded)) + encoded.hex()


def feature_frame(values):
    return varint_hex(len(values)) + "".join(feature_string_frame(value) for value in values)


def tag_values_frame(values):
    return varint_hex(len(values)) + "".join(varint_hex(value) for value in values)


def tags_frame(registry, entries):
    return (varint_hex(1) + feature_string_frame(registry) + varint_hex(len(entries))
            + "".join(feature_string_frame(tag) + tag_values_frame(values)
                       for tag, values in entries))
