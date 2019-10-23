#[cfg(test)]
mod tests {
    use crate::{
        generation::{filename_to_render, RenderReturn, RenderType},
        instrument::oscillator::Basis,
        renderable::{
            calculate_fgpl, m_a_and_basis_to_f64, nf_to_vec_renderable,
            render_voice::renderables_to_render_voices, render_voice::RenderVoice, RenderOp,
        },
    };
    use num_rational::Rational64;
    use socool_ast::{OscType, PointOp};

    #[test]
    fn test_calculate_fgpl() {
        let basis = Basis {
            f: Rational64::new(2, 1),
            g: Rational64::new(1, 1),
            p: Rational64::new(0, 1),
            l: Rational64::new(1, 1),
            a: Rational64::new(1, 1),
            d: Rational64::new(1, 1),
        };
        let point_op = PointOp::init();
        let result = calculate_fgpl(&basis, &point_op);
        let expected = (2.0, (0.5, 0.5), 0.0, 1.0);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_m_a_and_basis_to_f64() {
        let result = m_a_and_basis_to_f64(
            Rational64::new(2, 1),
            Rational64::new(300, 1),
            Rational64::new(4, 1),
        );
        let expected = 604.0;
        assert_eq!(result, expected);
    }

    #[test]
    fn test_nf_to_vec_renderable() {
        let (nf, basis, table) = match filename_to_render(
            &"songs/test/render_op.socool".to_string(),
            RenderType::NfBasisAndTable,
        )
        .unwrap()
        {
            RenderReturn::NfBasisAndTable(nf, basis, table) => (nf, basis, table),
            _ => {
                panic!("missing songs/tests/render_op.socool");
            }
        };
        //dbg!(nf);
        let result = nf_to_vec_renderable(&nf, &table, &basis);
        let expected: Vec<Vec<RenderOp>> = vec![
            vec![RenderOp {
                f: 220.0,
                p: 0.0,
                g: (0.5, 0.5),
                l: 1.0,
                t: 0.0,
                attack: 44_100.0,
                decay: 44_100.0,
                decay_length: 2,
                samples: 44100,
                total_samples: 44100,
                index: 0,
                voice: 0,
                event: 0,
                portamento: 1.0,
                osc_type: OscType::Sine,
                next_l_silent: false,
                next_r_silent: false,
            }],
            vec![RenderOp {
                f: 330.0,
                p: 0.4,
                l: 1.0,
                g: (0.7, 0.3),
                t: 0.0,
                attack: 44_100.0,
                decay: 44_100.0,
                decay_length: 2,
                samples: 44100,
                total_samples: 44100,
                index: 0,
                voice: 1,
                event: 0,
                portamento: 1.0,
                osc_type: OscType::Sine,
                next_l_silent: false,
                next_r_silent: false,
            }],
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_get_batch() {
        let filename = "songs/test/render_op_get_batch.socool".to_string();
        let (nf, basis, table) =
            match filename_to_render(&filename, RenderType::NfBasisAndTable).unwrap() {
                RenderReturn::NfBasisAndTable(nf, basis, table) => (nf, basis, table),
                _ => {
                    panic!();
                }
            };
        let renderables = nf_to_vec_renderable(&nf, &table, &basis);
        let voices = renderables_to_render_voices(renderables);
        let mut voice = voices[0].clone();
        //Two ops each with 44_100 samples
        //Use everything but the last 100 samples of the first op
        let batch = voice.get_batch(44_000, None);
        assert_eq!(batch.len(), 1);
        //Use the rest of the first op and start the second op;
        let batch = voice.get_batch(200, None);
        assert_eq!(batch.len(), 2);

        assert_eq!(batch[0].samples, 100);
        assert_eq!(batch[0].index, 44_000);
        assert_eq!(batch[0].f, 220.0);

        assert_eq!(batch[1].samples, 100);
        assert_eq!(batch[1].index, 0);
        assert_eq!(batch[1].f, 247.5);

        //let _ = voice.get_batch(44_000, None);
        //let batch = voice.get_batch(200, None);

        //Expect the voice to wrap around when it runs out of ops
        //assert_eq!(batch[0].samples, 200);
        //assert_eq!(batch[0].index, 0);
        //assert_eq!(batch[0].f, 220.0);
    }
}
