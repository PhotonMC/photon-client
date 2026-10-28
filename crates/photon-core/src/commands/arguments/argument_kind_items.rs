use super::*;

impl ArgumentKind {
    /// `ArgumentType.parse`: read the argument off the reader, for a source
    /// that may use entity selectors.
    pub fn read(&self, reader: &mut StringReader) -> Read {
        self.read_as(reader, true)
    }

    /// `ArgumentType.parse(reader, source)`: the source's permission to use
    /// entity selectors is the one thing a kind reads from it.
    pub fn read_as(&self, reader: &mut StringReader, selectors: bool) -> Read {
        match self {
            Self::Bool => reader.read_boolean().map(drop),
            Self::Integer { min, max } => {
                let start = reader.cursor();
                let value = reader.read_int()?;
                if value < *min {
                    reader.set_cursor(start);
                    return Err(range_error(
                        reader,
                        "integer",
                        "low",
                        min,
                        value.to_string(),
                    ));
                }
                if value > *max {
                    reader.set_cursor(start);
                    return Err(range_error(
                        reader,
                        "integer",
                        "big",
                        max,
                        value.to_string(),
                    ));
                }
                Ok(())
            }
            Self::Long { min, max } => {
                let start = reader.cursor();
                let value = reader.read_long()?;
                if value < *min {
                    reader.set_cursor(start);
                    return Err(range_error(reader, "long", "low", min, value.to_string()));
                }
                if value > *max {
                    reader.set_cursor(start);
                    return Err(range_error(reader, "long", "big", max, value.to_string()));
                }
                Ok(())
            }
            Self::Float { min, max } => {
                let start = reader.cursor();
                let value = reader.read_float()?;
                if value < *min {
                    reader.set_cursor(start);
                    return Err(range_error(
                        reader,
                        "float",
                        "low",
                        java_float(*min),
                        java_float(value),
                    ));
                }
                if value > *max {
                    reader.set_cursor(start);
                    return Err(range_error(
                        reader,
                        "float",
                        "big",
                        java_float(*max),
                        java_float(value),
                    ));
                }
                Ok(())
            }
            Self::Double { min, max } => {
                let start = reader.cursor();
                let value = reader.read_double()?;
                if value < *min {
                    reader.set_cursor(start);
                    return Err(range_error(
                        reader,
                        "double",
                        "low",
                        java_double(*min),
                        java_double(value),
                    ));
                }
                if value > *max {
                    reader.set_cursor(start);
                    return Err(range_error(
                        reader,
                        "double",
                        "big",
                        java_double(*max),
                        java_double(value),
                    ));
                }
                Ok(())
            }
            Self::String(StringType::Word) | Self::Unquoted => {
                reader.read_unquoted();
                Ok(())
            }
            Self::String(StringType::Phrase) => reader.read_string().map(drop),
            Self::String(StringType::Greedy) | Self::Greedy => {
                reader.set_cursor(reader.total_length());
                Ok(())
            }
            Self::BlockPos => {
                if reader.can_read() && reader.peek() == '^' {
                    local_coordinates(reader)
                } else {
                    coordinates(reader, 3, "argument.pos3d.incomplete", coordinate_int)
                }
            }
            Self::Vec3 => {
                if reader.can_read() && reader.peek() == '^' {
                    local_coordinates(reader)
                } else {
                    coordinates(reader, 3, "argument.pos3d.incomplete", coordinate_double)
                }
            }
            Self::ColumnPos => {
                if !reader.can_read() {
                    return Err(CommandSyntaxException::at(
                        reader,
                        "argument.pos2d.incomplete",
                        &[],
                    ));
                }
                coordinates(reader, 2, "argument.pos2d.incomplete", coordinate_int)
            }
            Self::Vec2 | Self::Rotation => {
                if !reader.can_read() {
                    return Err(CommandSyntaxException::at(
                        reader,
                        "argument.pos2d.incomplete",
                        &[],
                    ));
                }
                coordinates(reader, 2, "argument.pos2d.incomplete", coordinate_double)
            }
            Self::Angle => {
                if !reader.can_read() {
                    return Err(CommandSyntaxException::at(
                        reader,
                        "argument.angle.incomplete",
                        &[],
                    ));
                }
                relative(reader);
                if reader.can_read() && reader.peek() != ' ' {
                    let value = reader.read_float()?;
                    if !value.is_finite() {
                        return Err(CommandSyntaxException::at(
                            reader,
                            "argument.angle.invalid",
                            &[],
                        ));
                    }
                }
                Ok(())
            }
            Self::Identifier => identifier(reader),
            Self::GameMode => one_of(
                reader,
                &["survival", "creative", "adventure", "spectator"],
                "argument.gamemode.invalid",
                false,
            ),
            Self::EntityAnchor => {
                one_of(reader, &["feet", "eyes"], "argument.anchor.invalid", true)
            }
            Self::ScoreboardSlot => {
                // The slot names hold dots, which `readUnquotedString` takes.
                one_of(
                    reader,
                    &crate::scoreboard::DISPLAY_SLOT_NAMES,
                    "argument.scoreboardDisplaySlot.invalid",
                    false,
                )
            }
            Self::TeamColor => one_of(
                reader,
                &crate::scoreboard::TEAM_COLOURS,
                "argument.color.invalid",
                false,
            ),
            Self::HexColor => {
                let text = reader.read_unquoted();
                if matches!(text.chars().count(), 3 | 6) {
                    Ok(())
                } else {
                    Err(CommandSyntaxException::at(
                        reader,
                        "argument.hexcolor.invalid",
                        &[literal(text)],
                    ))
                }
            }
            Self::Operation => {
                if !reader.can_read() {
                    return Err(CommandSyntaxException::at(
                        reader,
                        "arguments.operation.invalid",
                        &[],
                    ));
                }
                let start = reader.cursor();
                word(reader);
                let text = reader.slice(start, reader.cursor());
                if ["=", "+=", "-=", "*=", "/=", "%=", "<", ">", "><"].contains(&text.as_str()) {
                    Ok(())
                } else {
                    Err(CommandSyntaxException::plain(
                        "arguments.operation.invalid",
                        &[],
                    ))
                }
            }
            Self::Swizzle => {
                let mut seen = String::new();
                while reader.can_read() && reader.peek() != ' ' {
                    let c = reader.read();
                    if !matches!(c, 'x' | 'y' | 'z') || seen.contains(c) {
                        return Err(CommandSyntaxException::at(
                            reader,
                            "arguments.swizzle.invalid",
                            &[],
                        ));
                    }
                    seen.push(c);
                }
                Ok(())
            }
            Self::Uuid => {
                let rest = reader.remaining();
                let end = rest
                    .chars()
                    .take_while(|c| c.is_ascii_hexdigit() || *c == '-')
                    .count();
                let candidate: String = rest.chars().take(end).collect();
                if uuid_valid(&candidate) {
                    reader.set_cursor(reader.cursor() + end);
                    Ok(())
                } else {
                    Err(CommandSyntaxException::at(
                        reader,
                        "argument.uuid.invalid",
                        &[],
                    ))
                }
            }
            Self::Time { min } => {
                let value = reader.read_float()?;
                let unit = reader.read_unquoted();
                let factor = match unit.as_str() {
                    "d" => 24000,
                    "s" => 20,
                    "t" | "" => 1,
                    _ => {
                        return Err(CommandSyntaxException::at(
                            reader,
                            "argument.time.invalid_unit",
                            &[],
                        ));
                    }
                };
                let ticks = (value * factor as f32 + 0.5).floor() as i32;
                if ticks < *min {
                    Err(CommandSyntaxException::at(
                        reader,
                        "argument.time.tick_count_too_low",
                        &[literal(min.to_string()), literal(ticks.to_string())],
                    ))
                } else {
                    Ok(())
                }
            }
            Self::Entity {
                single,
                players_only,
            } => entity(reader, *single, *players_only, selectors),
            Self::Word => {
                word(reader);
                Ok(())
            }
            Self::Balanced => {
                balanced(reader);
                Ok(())
            }
            Self::ScoreHolder => {
                if !selectors && reader.can_read() && reader.peek() == '@' {
                    return Err(CommandSyntaxException::at(
                        reader,
                        "argument.entity.selector.not_allowed",
                        &[],
                    ));
                }
                balanced(reader);
                Ok(())
            }
        }
    }
}
