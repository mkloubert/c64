// Cobra64 - A concept for a modern Python-like compiler creating C64 binaries
//
// Copyright (C) 2026 Marcel Joachim Kloubert <marcel@kloubert.dev>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Expression code generation.
//!
//! This module provides code generation for all expression types:
//! - Literals (integer, bool, char, string, fixed, float)
//! - Identifiers (variable access)
//! - Binary and unary operations
//! - Function calls
//! - Type casts
//! - Array indexing

use super::binary_ops::BinaryOpsEmitter;
use super::conversions::TypeConversions;
use super::data_blocks::DataBlockEmitter;
use super::emit::EmitHelpers;
use super::functions::FunctionCallEmitter;
use super::mos6510::{c64, colors, opcodes, sid, sprite, vic, zeropage};
use super::strings::StringManager;
use super::type_inference::TypeInference;
use super::types::{decimal_string_to_binary16, decimal_string_to_fixed};
use super::unary_ops::UnaryOpsEmitter;
use super::variables::VariableManager;
use super::CodeGenerator;
use crate::ast::{Expr, ExprKind, Type};
use crate::error::{CompileError, ErrorCode};

/// Get the value of a built-in constant, if it exists.
/// Returns Some((value, is_word)) for known constants.
fn get_builtin_constant(name: &str) -> Option<(u16, bool)> {
    match name {
        // C64 color constants (byte values)
        "COLOR_BLACK" => Some((colors::BLACK as u16, false)),
        "COLOR_WHITE" => Some((colors::WHITE as u16, false)),
        "COLOR_RED" => Some((colors::RED as u16, false)),
        "COLOR_CYAN" => Some((colors::CYAN as u16, false)),
        "COLOR_PURPLE" => Some((colors::PURPLE as u16, false)),
        "COLOR_GREEN" => Some((colors::GREEN as u16, false)),
        "COLOR_BLUE" => Some((colors::BLUE as u16, false)),
        "COLOR_YELLOW" => Some((colors::YELLOW as u16, false)),
        "COLOR_ORANGE" => Some((colors::ORANGE as u16, false)),
        "COLOR_BROWN" => Some((colors::BROWN as u16, false)),
        "COLOR_LIGHT_RED" => Some((colors::LIGHT_RED as u16, false)),
        "COLOR_DARK_GRAY" => Some((colors::DARK_GRAY as u16, false)),
        "COLOR_GRAY" => Some((colors::GRAY as u16, false)),
        "COLOR_LIGHT_GREEN" => Some((colors::LIGHT_GREEN as u16, false)),
        "COLOR_LIGHT_BLUE" => Some((colors::LIGHT_BLUE as u16, false)),
        "COLOR_LIGHT_GRAY" => Some((colors::LIGHT_GRAY as u16, false)),
        // VIC-II sprite registers (word values - memory addresses)
        "VIC_SPRITE_ENABLE" => Some((sprite::ENABLE, true)),
        "VIC_SPRITE_X_MSB" => Some((sprite::X_MSB, true)),
        "VIC_SPRITE_EXPAND_Y" => Some((sprite::EXPAND_Y, true)),
        "VIC_SPRITE_PRIORITY" => Some((sprite::PRIORITY, true)),
        "VIC_SPRITE_MULTICOLOR" => Some((sprite::MULTICOLOR, true)),
        "VIC_SPRITE_EXPAND_X" => Some((sprite::EXPAND_X, true)),
        "VIC_SPRITE_COLLISION_SPRITE" => Some((sprite::COLLISION_SPRITE, true)),
        "VIC_SPRITE_COLLISION_BG" => Some((sprite::COLLISION_BG, true)),
        "VIC_SPRITE_MULTICOLOR1" => Some((sprite::MULTICOLOR1, true)),
        "VIC_SPRITE_MULTICOLOR2" => Some((sprite::MULTICOLOR2, true)),
        "VIC_SPRITE_POINTER_BASE" => Some((sprite::POINTER_BASE, true)),
        // Individual sprite position registers
        "VIC_SPRITE0_X" => Some((sprite::SPRITE0_X, true)),
        "VIC_SPRITE0_Y" => Some((sprite::SPRITE0_Y, true)),
        "VIC_SPRITE1_X" => Some((sprite::SPRITE1_X, true)),
        "VIC_SPRITE1_Y" => Some((sprite::SPRITE1_Y, true)),
        "VIC_SPRITE2_X" => Some((sprite::SPRITE2_X, true)),
        "VIC_SPRITE2_Y" => Some((sprite::SPRITE2_Y, true)),
        "VIC_SPRITE3_X" => Some((sprite::SPRITE3_X, true)),
        "VIC_SPRITE3_Y" => Some((sprite::SPRITE3_Y, true)),
        "VIC_SPRITE4_X" => Some((sprite::SPRITE4_X, true)),
        "VIC_SPRITE4_Y" => Some((sprite::SPRITE4_Y, true)),
        "VIC_SPRITE5_X" => Some((sprite::SPRITE5_X, true)),
        "VIC_SPRITE5_Y" => Some((sprite::SPRITE5_Y, true)),
        "VIC_SPRITE6_X" => Some((sprite::SPRITE6_X, true)),
        "VIC_SPRITE6_Y" => Some((sprite::SPRITE6_Y, true)),
        "VIC_SPRITE7_X" => Some((sprite::SPRITE7_X, true)),
        "VIC_SPRITE7_Y" => Some((sprite::SPRITE7_Y, true)),
        // Individual sprite color registers
        "VIC_SPRITE0_COLOR" => Some((sprite::SPRITE0_COLOR, true)),
        "VIC_SPRITE1_COLOR" => Some((sprite::SPRITE1_COLOR, true)),
        "VIC_SPRITE2_COLOR" => Some((sprite::SPRITE2_COLOR, true)),
        "VIC_SPRITE3_COLOR" => Some((sprite::SPRITE3_COLOR, true)),
        "VIC_SPRITE4_COLOR" => Some((sprite::SPRITE4_COLOR, true)),
        "VIC_SPRITE5_COLOR" => Some((sprite::SPRITE5_COLOR, true)),
        "VIC_SPRITE6_COLOR" => Some((sprite::SPRITE6_COLOR, true)),
        "VIC_SPRITE7_COLOR" => Some((sprite::SPRITE7_COLOR, true)),

        // SID base address
        "SID_BASE" => Some((sid::BASE, true)),

        // SID waveform constants (byte values)
        "WAVE_TRIANGLE" => Some((sid::WAVEFORM_TRIANGLE as u16, false)),
        "WAVE_SAWTOOTH" => Some((sid::WAVEFORM_SAWTOOTH as u16, false)),
        "WAVE_PULSE" => Some((sid::WAVEFORM_PULSE as u16, false)),
        "WAVE_NOISE" => Some((sid::WAVEFORM_NOISE as u16, false)),

        // SID filter mode constants (byte values)
        "FILTER_LOWPASS" => Some((sid::FILTER_LP as u16, false)),
        "FILTER_BANDPASS" => Some((sid::FILTER_BP as u16, false)),
        "FILTER_HIGHPASS" => Some((sid::FILTER_HP as u16, false)),

        // Musical note constants (byte values, 0-11)
        "NOTE_C" => Some((0, false)),
        "NOTE_CS" => Some((1, false)),
        "NOTE_D" => Some((2, false)),
        "NOTE_DS" => Some((3, false)),
        "NOTE_E" => Some((4, false)),
        "NOTE_F" => Some((5, false)),
        "NOTE_FS" => Some((6, false)),
        "NOTE_G" => Some((7, false)),
        "NOTE_GS" => Some((8, false)),
        "NOTE_A" => Some((9, false)),
        "NOTE_AS" => Some((10, false)),
        "NOTE_B" => Some((11, false)),

        // SID Voice 1 register addresses
        "SID_VOICE1_FREQ_LO" => Some((sid::VOICE1_FREQ_LO, true)),
        "SID_VOICE1_FREQ_HI" => Some((sid::VOICE1_FREQ_HI, true)),
        "SID_VOICE1_PULSE_LO" => Some((sid::VOICE1_PULSE_LO, true)),
        "SID_VOICE1_PULSE_HI" => Some((sid::VOICE1_PULSE_HI, true)),
        "SID_VOICE1_CTRL" => Some((sid::VOICE1_CTRL, true)),
        "SID_VOICE1_AD" => Some((sid::VOICE1_ATTACK_DECAY, true)),
        "SID_VOICE1_SR" => Some((sid::VOICE1_SUSTAIN_RELEASE, true)),

        // SID Voice 2 register addresses
        "SID_VOICE2_FREQ_LO" => Some((sid::VOICE2_FREQ_LO, true)),
        "SID_VOICE2_FREQ_HI" => Some((sid::VOICE2_FREQ_HI, true)),
        "SID_VOICE2_PULSE_LO" => Some((sid::VOICE2_PULSE_LO, true)),
        "SID_VOICE2_PULSE_HI" => Some((sid::VOICE2_PULSE_HI, true)),
        "SID_VOICE2_CTRL" => Some((sid::VOICE2_CTRL, true)),
        "SID_VOICE2_AD" => Some((sid::VOICE2_ATTACK_DECAY, true)),
        "SID_VOICE2_SR" => Some((sid::VOICE2_SUSTAIN_RELEASE, true)),

        // SID Voice 3 register addresses
        "SID_VOICE3_FREQ_LO" => Some((sid::VOICE3_FREQ_LO, true)),
        "SID_VOICE3_FREQ_HI" => Some((sid::VOICE3_FREQ_HI, true)),
        "SID_VOICE3_PULSE_LO" => Some((sid::VOICE3_PULSE_LO, true)),
        "SID_VOICE3_PULSE_HI" => Some((sid::VOICE3_PULSE_HI, true)),
        "SID_VOICE3_CTRL" => Some((sid::VOICE3_CTRL, true)),
        "SID_VOICE3_AD" => Some((sid::VOICE3_ATTACK_DECAY, true)),
        "SID_VOICE3_SR" => Some((sid::VOICE3_SUSTAIN_RELEASE, true)),

        // SID filter and volume registers
        "SID_FILTER_CUTOFF_LO" => Some((sid::FILTER_CUTOFF_LO, true)),
        "SID_FILTER_CUTOFF_HI" => Some((sid::FILTER_CUTOFF_HI, true)),
        "SID_FILTER_RESONANCE" => Some((sid::FILTER_RESONANCE, true)),
        "SID_VOLUME" => Some((sid::FILTER_MODE_VOLUME, true)),

        // =====================================================================
        // VIC-II Graphics Registers
        // =====================================================================

        // Control registers
        "VIC_CONTROL1" => Some((vic::CONTROL1, true)),
        "VIC_CONTROL2" => Some((vic::CONTROL2, true)),
        "VIC_MEMORY" => Some((vic::MEMORY, true)),
        "VIC_RASTER" => Some((vic::RASTER, true)),

        // Color registers
        "VIC_BORDER" => Some((vic::BORDER, true)),
        "VIC_BACKGROUND" => Some((vic::BACKGROUND0, true)),
        "VIC_BACKGROUND0" => Some((vic::BACKGROUND0, true)),
        "VIC_BACKGROUND1" => Some((vic::BACKGROUND1, true)),
        "VIC_BACKGROUND2" => Some((vic::BACKGROUND2, true)),
        "VIC_BACKGROUND3" => Some((vic::BACKGROUND3, true)),

        // Color RAM constant
        "COLOR_RAM" => Some((c64::COLOR_RAM, true)),

        // Graphics mode constants (byte values)
        "GFX_TEXT" => Some((vic::MODE_TEXT as u16, false)),
        "GFX_TEXT_MC" => Some((vic::MODE_TEXT_MC as u16, false)),
        "GFX_BITMAP" => Some((vic::MODE_BITMAP as u16, false)),
        "GFX_BITMAP_MC" => Some((vic::MODE_BITMAP_MC as u16, false)),
        "GFX_TEXT_ECM" => Some((vic::MODE_TEXT_ECM as u16, false)),

        // VIC bank constants (byte values)
        "VIC_BANK0" => Some((vic::BANK0 as u16, false)),
        "VIC_BANK1" => Some((vic::BANK1 as u16, false)),
        "VIC_BANK2" => Some((vic::BANK2 as u16, false)),
        "VIC_BANK3" => Some((vic::BANK3 as u16, false)),

        // Raster constants (word values)
        "RASTER_TOP" => Some((vic::RASTER_TOP, true)),
        "RASTER_BOTTOM" => Some((vic::RASTER_BOTTOM, true)),
        "RASTER_MAX_PAL" => Some((vic::RASTER_MAX_PAL, true)),
        "RASTER_MAX_NTSC" => Some((vic::RASTER_MAX_NTSC, true)),

        _ => None,
    }
}

/// Extension trait for expression code generation.
pub trait ExpressionEmitter {
    /// Generate code for an expression.
    ///
    /// Result is left in A register (for byte) or A/X (for word, A=low, X=high).
    fn generate_expression(&mut self, expr: &Expr) -> Result<(), CompileError>;

    /// Generate code for an expression with a known target type.
    ///
    /// This is used when assigning to variables where the target type affects
    /// how literals (especially DecimalLiteral) should be interpreted.
    fn generate_expression_for_type(
        &mut self,
        expr: &Expr,
        target_type: &Type,
    ) -> Result<(), CompileError>;
}

impl ExpressionEmitter for CodeGenerator {
    fn generate_expression_for_type(
        &mut self,
        expr: &Expr,
        target_type: &Type,
    ) -> Result<(), CompileError> {
        // Handle DecimalLiteral specially based on target type
        if let ExprKind::DecimalLiteral(s) = &expr.kind {
            if target_type.is_fixed() {
                // Convert to fixed-point 12.4 format
                let value = decimal_string_to_fixed(s);
                self.emit_imm(opcodes::LDA_IMM, (value & 0xFF) as u8);
                self.emit_imm(opcodes::LDX_IMM, ((value >> 8) & 0xFF) as u8);
                return Ok(());
            }
            // For float or other types, fall through to default handling
        }
        // For all other cases, use standard expression generation
        self.generate_expression(expr)
    }

    fn generate_expression(&mut self, expr: &Expr) -> Result<(), CompileError> {
        match &expr.kind {
            ExprKind::IntegerLiteral(value) => {
                self.emit_imm(opcodes::LDA_IMM, (*value & 0xFF) as u8);
                if *value > 255 {
                    self.emit_imm(opcodes::LDX_IMM, (*value >> 8) as u8);
                }
            }
            ExprKind::BoolLiteral(value) => {
                self.emit_imm(opcodes::LDA_IMM, if *value { 1 } else { 0 });
            }
            ExprKind::CharLiteral(c) => {
                self.emit_imm(opcodes::LDA_IMM, *c as u8);
            }
            ExprKind::StringLiteral(s) => {
                let string_index = self.add_string(s);
                self.emit_string_ref(string_index);
            }
            ExprKind::Identifier(name) => {
                // Check for built-in constants first
                if let Some((value, is_word)) = get_builtin_constant(name) {
                    self.emit_imm(opcodes::LDA_IMM, (value & 0xFF) as u8);
                    if is_word {
                        self.emit_imm(opcodes::LDX_IMM, ((value >> 8) & 0xFF) as u8);
                    }
                } else if self.is_data_block(name) {
                    // Data block - emit reference to its address (word)
                    // The actual address will be patched after data blocks are emitted
                    self.emit_data_block_ref(name);
                } else {
                    let var = self.get_variable(name).ok_or_else(|| {
                        CompileError::new(
                            ErrorCode::UndefinedVariable,
                            format!("Undefined variable '{}'", name),
                            expr.span.clone(),
                        )
                    })?;
                    self.emit_load_from_address(var.address, &var.var_type);
                }
            }
            ExprKind::BinaryOp { left, op, right } => {
                self.generate_binary_op(left, *op, right)?;
            }
            ExprKind::UnaryOp { op, operand } => {
                self.generate_unary_op(*op, operand)?;
            }
            ExprKind::FunctionCall { name, args } => {
                self.generate_function_call(name, args, &expr.span)?;
            }
            ExprKind::TypeCast {
                target_type,
                expr: inner,
            } => {
                let source_type = self.infer_type_from_expr(inner);
                self.generate_expression(inner)?;
                self.generate_type_conversion(&source_type, target_type)?;
            }
            ExprKind::ArrayIndex { array, index } => {
                // For array index, we need to get the base address
                // The array expression should be an identifier
                if let ExprKind::Identifier(name) = &array.kind {
                    let var = self.get_variable(name).ok_or_else(|| {
                        CompileError::new(
                            ErrorCode::UndefinedVariable,
                            format!("Undefined array '{}'", name),
                            expr.span.clone(),
                        )
                    })?;

                    let is_word_array = matches!(
                        var.var_type,
                        Type::WordArray(_)
                            | Type::SwordArray(_)
                            | Type::FixedArray(_)
                            | Type::FloatArray(_)
                    );

                    // Generate index expression
                    self.generate_expression(index)?;

                    if is_word_array {
                        // For word arrays, multiply index by 2 (ASL A)
                        self.emit_byte(opcodes::ASL_ACC);
                    }

                    self.emit_byte(opcodes::TAY); // Y = index (or index*2 for word arrays)

                    // Load base address into TMP1
                    self.emit_imm(opcodes::LDA_IMM, (var.address & 0xFF) as u8);
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    self.emit_imm(opcodes::LDA_IMM, (var.address >> 8) as u8);
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP1_HI);

                    if is_word_array {
                        // Load 16-bit value: low byte to A, high byte to X
                        self.emit_byte(opcodes::LDA_IZY);
                        self.emit_byte(zeropage::TMP1);
                        self.emit_byte(opcodes::PHA); // Save low byte
                        self.emit_byte(opcodes::INY); // Point to high byte
                        self.emit_byte(opcodes::LDA_IZY);
                        self.emit_byte(zeropage::TMP1);
                        self.emit_byte(opcodes::TAX); // X = high byte
                        self.emit_byte(opcodes::PLA); // A = low byte
                    } else {
                        // Load 8-bit value (byte, bool)
                        self.emit_byte(opcodes::LDA_IZY);
                        self.emit_byte(zeropage::TMP1);
                    }
                } else {
                    return Err(CompileError::new(
                        ErrorCode::InvalidAssignmentTarget,
                        "Array index must be on an identifier",
                        expr.span.clone(),
                    ));
                }
            }
            ExprKind::FixedLiteral(value) => {
                // 16-bit fixed-point 12.4: store low byte in A, high byte in X
                self.emit_imm(opcodes::LDA_IMM, (*value & 0xFF) as u8);
                self.emit_imm(opcodes::LDX_IMM, ((*value >> 8) & 0xFF) as u8);
            }
            ExprKind::FloatLiteral(bits) => {
                // 16-bit IEEE-754 binary16: store low byte in A, high byte in X
                self.emit_imm(opcodes::LDA_IMM, (*bits & 0xFF) as u8);
                self.emit_imm(opcodes::LDX_IMM, ((*bits >> 8) & 0xFF) as u8);
            }
            ExprKind::DecimalLiteral(s) => {
                // Convert decimal string to IEEE-754 binary16 (default type)
                let bits = decimal_string_to_binary16(s);
                self.emit_imm(opcodes::LDA_IMM, (bits & 0xFF) as u8);
                self.emit_imm(opcodes::LDX_IMM, ((bits >> 8) & 0xFF) as u8);
            }
            ExprKind::Grouped(inner) => {
                self.generate_expression(inner)?;
            }
            ExprKind::ArrayLiteral { .. } => {
                // Array literals are handled during variable initialization.
                // If we reach here, it means the array literal is used in an
                // expression context that isn't supported yet.
                return Err(CompileError::new(
                    ErrorCode::NotImplemented,
                    "Array literals in expression context not yet implemented",
                    expr.span.clone(),
                ));
            }
        }
        Ok(())
    }
}
