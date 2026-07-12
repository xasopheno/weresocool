//! AST for the surface DSL. Mirrors `warp::ast` in shape — a pipeline is a
//! `source` followed by an ordered list of `ops`.

/// Where the surface geometry starts. Each variant produces a complete
/// initial mesh (positions + uvs + indices); subsequent ops mutate that
/// mesh in place.
#[derive(Debug, Clone, PartialEq)]
pub enum SurfaceSource {
    /// Subdivided flat plane in the XY plane (Z = 0 before any deformation).
    /// `width`/`height` are world units; `subdivs` is the number of segments
    /// per side (so vertex count = (subdivs+1)²).
    Plane { width: f32, height: f32, subdivs: u32 },
    /// Icosphere — radius + subdivision level. Level 0 = 12 verts; each
    /// subsequent level multiplies face count ~4×. UVs are spherical
    /// (longitude/latitude), useful with Triplanar for seamless wrap.
    Sphere { radius: f32, subdivs: u32 },
    /// Open cylinder (no caps), Z axis aligned. `subdivs` is the angular
    /// segment count; vertex rings are at top and bottom.
    Cylinder { radius: f32, height: f32, subdivs: u32 },
}

/// Which world axis an op operates around / along.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SurfaceAxis { X, Y, Z }

/// A vertex-level transformation applied to the mesh produced by previous
/// ops. Order matters: `Wave2 | Bend` produces different geometry than
/// `Bend | Wave2`.
#[derive(Debug, Clone, PartialEq)]
pub enum SurfaceOp {
    /// 2D sine displacement perpendicular to the source's primary face.
    /// For a Plane, this displaces Z; for a Sphere/Cylinder, this displaces
    /// along the local surface normal (radially outward).
    ///
    /// `z = amp · sin(fx · x) · cos(fy · y)` (Plane variant). `fx`, `fy`
    /// are spatial frequencies in radians per world unit.
    Wave2 { fx: f32, fy: f32, amp: f32 },
    /// Multi-octave value noise displacement along the surface normal.
    /// `scale` controls the noise's spatial frequency (higher = more detail);
    /// `amp` controls displacement magnitude in world units.
    Noise { scale: f32, amp: f32 },
    /// Bend the mesh around an axis by `angle` (degrees). Each vertex is
    /// rotated by an angle proportional to its distance along the axis,
    /// so the surface curls into a partial-cylinder shape. Useful for
    /// turning a flat plane into a curved IMAX-style canvas.
    Bend { axis: SurfaceAxis, angle_deg: f32 },
    /// Twist the mesh around an axis by `strength` (radians per world unit
    /// along the axis). Each vertex is rotated by `strength · axis_pos`
    /// around the axis. Turns a strip into a helical ribbon.
    Twist { axis: SurfaceAxis, strength: f32 },
    /// Recompute smooth per-vertex normals from the current geometry.
    /// Required after any displacement op if the PBR material reads
    /// normals for shading (otherwise normals carry stale pre-deformation
    /// orientation and shading looks wrong).
    Smooth,
}

/// A surface authored in the DSL — a source plus an ordered list of ops.
#[derive(Debug, Clone, PartialEq)]
pub struct SurfacePipeline {
    pub source: SurfaceSource,
    pub ops: Vec<SurfaceOp>,
}

/// A named surface definition extracted from a .socool file.
#[derive(Debug, Clone, PartialEq)]
pub struct SurfaceDef {
    pub name: String,
    pub pipeline: SurfacePipeline,
}
