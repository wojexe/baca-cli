use crate::model::Submit;
use crate::model::SubmitStatus;
use crate::model::TestResults;
use crate::workspace::ConnectionConfig;
use std::cmp;
use std::str::FromStr;

impl Submit {
    pub fn parse(connection_config: &ConnectionConfig, data: &str) -> Submit {
        let data = Self::deserialize(data);
        tracing::debug!("Deserialized: {:?}", data);

        let test_data = Submit::parse_test_statuses(&data);
        let mut submit = Submit::parse_submit_info(connection_config, data);
        submit.test_results = test_data;
        submit
    }

    fn parse_submit_info(connection_config: &ConnectionConfig, data: Vec<String>) -> Submit {
        let st: Vec<_> = data
            .iter()
            .skip_while(|x| !x.contains("nazwa statusu"))
            .map(|x| x.replace("\"", ""))
            .collect();

        let offset = 10;
        Submit {
            status: SubmitStatus::from_str(&*st[offset]).unwrap(),
            points: st[offset + 1].parse().unwrap(),
            lateness: Some(st[offset + 2].parse().unwrap()),
            accepted: st[offset + 3].parse().unwrap(),
            size: st[offset + 4].parse().unwrap(),
            timestamp: st[offset + 5].to_string(),
            language: st[offset + 6].to_string(),
            id: st[offset + 7].to_string(),
            max_points: Some(st[offset + 7 + 25].parse().unwrap()),
            problem_name: st[offset + 7 + 26].to_string(),
            link: connection_config.make_url() + "/#SubmitDetails/" + st[offset + 7].as_str(),
            test_results: None,
        }
    }

    fn parse_test_statuses(data: &[String]) -> Option<Vec<TestResults>> {
        let raw_test_data = Submit::collect_tests_data(data);
        let (offset, test_name_idx, test_status_idx) =
            Submit::find_name_and_status_indices(&raw_test_data);
        let raw_test_data = raw_test_data
            .iter()
            .filter(|x| !x.contains("lang.String"))
            .collect::<Vec<_>>();

        let mut ans = raw_test_data
            .chunks(offset)
            .into_iter()
            .filter(|x| x.len() == offset)
            .skip(1)
            .map(|data| TestResults {
                name: data[test_name_idx].to_string(),
                status: SubmitStatus::from_str(&*data[test_status_idx])
                    .expect("Invalid test status"),
            })
            .into_iter()
            .collect::<Vec<_>>();
        ans.reverse();
        if ans.is_empty() {
            None
        } else {
            Some(ans)
        }
    }

    fn find_name_and_status_indices(raw_test_data: &[String]) -> (usize, usize, usize) {
        let mut offset = 0;
        let mut test_name_idx = usize::MAX;
        let mut test_status_idx = usize::MAX;
        for x in raw_test_data {
            if x.contains("test") {
                test_name_idx = cmp::min(test_name_idx, offset);
            } else if x.contains("status") {
                test_status_idx = cmp::min(test_status_idx, offset);
            } else if x.contains("java.lang.String") {
                break;
            }
            offset += 1;
        }
        (offset, test_name_idx, test_status_idx)
    }

    fn collect_tests_data(data: &[String]) -> Vec<String> {
        data.iter()
            .take_while(|x| !x.contains("nazwa statusu"))
            .map(|x| x.replace("\"", ""))
            .collect()
    }

    fn deserialize(data: &str) -> Vec<String> {
        let data: String = data.chars().skip(5).take(data.len() - 13).collect();
        let data = data.split(',').collect::<Vec<&str>>();

        let encoded = data
            .iter()
            .take_while(|x| (**x).chars().all(|c| c.is_ascii_digit()))
            .collect::<Vec<&&str>>();

        let codes = data
            .iter()
            .skip(encoded.len())
            .map(|x| x.to_string())
            .collect::<Vec<String>>();
        let codes = codes.join(",");
        let codes = codes.split(",\"").collect::<Vec<&str>>();

        let mut deserialized = Vec::<String>::new();
        for l in encoded {
            let val = l.to_string().parse::<usize>().unwrap();

            if val == 0 {
                continue;
            }

            deserialized.push((*codes[val - 1]).to_string());
        }

        deserialized
    }
}

#[cfg(test)]
mod submit_parser_tests {
    use crate::model::SubmitStatus;
    use crate::model::{Submit, TestResults};
    use crate::workspace::ConnectionConfig;

    //todo: test code wih sequence "asda","asdasd"

    #[test]
    fn correct_p2_parse_test() {
        let baca = ConnectionConfig {
            host: "p22019".to_string(),
            login: "".to_string(),
            password: "".to_string(),
            permutation: "permutation".to_string(),
            cookie: "cookie".to_string(),
        };
        let raw = r#"//OK[0,19,68,2,5,46,67,2,5,7,66,2,5,7,65,2,5,58,64,2,5,46,63,2,5,58,62,2,5,58,61,2,5,58,60,2,5,46,59,2,5,58,57,2,5,46,56,2,5,7,55,2,5,12,4,3,0,54,53,52,51,50,49,48,47,8,5,46,45,44,43,42,41,40,39,8,5,1,4,3,38,0,37,36,35,34,33,32,31,30,29,9,5,28,27,26,25,24,23,22,21,20,9,5,1,4,3,0,0,19,18,2,5,7,17,2,5,7,16,2,5,7,15,2,5,7,14,2,5,7,13,2,5,7,12,2,5,7,11,2,5,7,6,2,5,7,10,2,5,7,9,2,5,7,8,2,5,7,6,2,5,12,4,3,2,1,["testerka.gwt.client.submits.SubmitDetailsModel/2564112456","compilation_logs","testerka.gwt.client.tools.DataSource/1474249525","[[Ljava.lang.String;/4182515373","[Ljava.lang.String;/2600011424","228","program zaakceptowany","388","204","424","244","248","436","252","192","284","1552","czas","status","F - Wielomiany","12","2019-05-15 00:00:00","2019-05-25 00:00:00","2019-06-01 00:00:00","122","30","1024","125","Nazwa zdania","Liczba punktow do zdobycia","Start zadania","Termin oddania","Koniec zdania","Limit pamieci (MB)","Limit czasu kompilacji (s)","Limit pamieci na kompilacje (MB)","Limit kodu zrodlowego (kB)","source_code","7998","C++","2019-05-16 12:04:18","1414","13","100","1.59","bĹ\x82Ä\x85d wykonania","id","język","czas zgłoszenia","rozmiar (b)","zaliczone (%)","spoznienie (%)","punkty","nazwa statusu","1_assign/assign","2_arthmetics/1_plus/plus","2_arthmetics/2_minus_unary/minus_unary","zĹ\x82a odpowiedz","2_arthmetics/3_minus_binary/minus_binary","2_arthmetics/4_asterisk/asterisk","2_arthmetics/5_slash_percent/slesh_percent","2_arthmetics/6_shifts/shifts","3_composites/composites","4_incr_decr/incr_decr","5_dynamic_memory/dynamic_memory","6_relationals/relationals","7_various/various","test"],0,7]"#;

        let actual = Submit::parse(&baca, raw);
        let expected = Submit {
            status: SubmitStatus::RuntimeError,
            points: 1.59,
            lateness: Some(100),
            accepted: 13,
            size: 1414,
            timestamp: "2019-05-16 12:04:18".to_string(),
            language: "C++".to_string(),
            id: "7998".to_string(),
            max_points: Some(12),
            problem_name: "F - Wielomiany".to_string(),
            link: "https://baca.ii.uj.edu.pl/p22019/#SubmitDetails/7998".to_string(),
            test_results: Some(vec![
                TestResults {
                    name: "1_assign/assign".to_string(),
                    status: SubmitStatus::Ok,
                },
                TestResults {
                    name: "2_arthmetics/1_plus/plus".to_string(),
                    status: SubmitStatus::RuntimeError,
                },
                TestResults {
                    name: "2_arthmetics/2_minus_unary/minus_unary".to_string(),
                    status: SubmitStatus::WrongAnswer,
                },
                TestResults {
                    name: "2_arthmetics/3_minus_binary/minus_binary".to_string(),
                    status: SubmitStatus::RuntimeError,
                },
                TestResults {
                    name: "2_arthmetics/4_asterisk/asterisk".to_string(),
                    status: SubmitStatus::WrongAnswer,
                },
                TestResults {
                    name: "2_arthmetics/5_slash_percent/slesh_percent".to_string(),
                    status: SubmitStatus::WrongAnswer,
                },
                TestResults {
                    name: "2_arthmetics/6_shifts/shifts".to_string(),
                    status: SubmitStatus::WrongAnswer,
                },
                TestResults {
                    name: "3_composites/composites".to_string(),
                    status: SubmitStatus::RuntimeError,
                },
                TestResults {
                    name: "4_incr_decr/incr_decr".to_string(),
                    status: SubmitStatus::WrongAnswer,
                },
                TestResults {
                    name: "5_dynamic_memory/dynamic_memory".to_string(),
                    status: SubmitStatus::Ok,
                },
                TestResults {
                    name: "6_relationals/relationals".to_string(),
                    status: SubmitStatus::Ok,
                },
                TestResults {
                    name: "7_various/various".to_string(),
                    status: SubmitStatus::RuntimeError,
                },
            ]),
        };

        actual.print_with_tests();
        assert_eq!(actual, expected);
    }

    #[test]
    fn correct_mn_parse_test() {
        let baca = ConnectionConfig {
            host: "mn".to_string(),
            login: "".to_string(),
            password: "".to_string(),
            permutation: "permutation".to_string(),
            cookie: "cookie".to_string(),
        };
        let raw = r#"//OK[0,12,50,2,5,7,49,2,5,7,48,2,5,7,47,2,5,7,46,2,5,4,4,3,0,45,44,43,42,41,40,39,38,8,5,7,37,36,36,35,34,33,32,8,5,1,4,3,31,0,30,29,28,27,26,25,24,23,22,9,5,21,20,19,18,17,16,15,14,13,9,5,1,4,3,0,0,12,11,2,5,7,10,2,5,7,9,2,5,7,8,2,5,7,6,2,5,4,4,3,2,1,["testerka.gwt.client.submits.SubmitDetailsModel/2564112456","logs","testerka.gwt.client.tools.DataSource/1474249525","[[Ljava.lang.String;/4182515373","[Ljava.lang.String;/2600011424","532","program zaakceptowany","536","564","572","czas","status","[G] Funkcje sklejane","4","2020-05-13 07:39:59","2020-06-04 23:00:00","2020-06-15 23:00:00","122","30","1024","125","Nazwa zdania","Liczba punktow do zdobycia","Start zadania","Termin oddania","Koniec zdania","Limit pamieci (MB)","Limit czasu kompilacji (s)","Limit pamieci na kompilacje (MB)","Limit kodu zrodlowego (kB)","code","4334","C++","2020-05-17 18:53:09","1190","100","4.00","id","język","czas zgłoszenia","rozmiar (b)","zaliczone (%)","spoznienie (%)","punkty","nazwa statusu","test0/0","test1/0","test2/0","test3/0","test"],0,7]"#;

        let actual = Submit::parse(&baca, raw);
        let expected = Submit {
            status: SubmitStatus::Ok,
            points: 4.0,
            lateness: Some(100),
            accepted: 100,
            size: 1190,
            timestamp: "2020-05-17 18:53:09".to_string(),
            language: "C++".to_string(),
            id: "4334".to_string(),
            max_points: Some(4),
            problem_name: "[G] Funkcje sklejane".to_string(),
            link: "https://baca.ii.uj.edu.pl/mn/#SubmitDetails/4334".to_string(),
            test_results: Some(vec![
                TestResults {
                    name: "test0/0".to_string(),
                    status: SubmitStatus::Ok,
                },
                TestResults {
                    name: "test1/0".to_string(),
                    status: SubmitStatus::Ok,
                },
                TestResults {
                    name: "test2/0".to_string(),
                    status: SubmitStatus::Ok,
                },
                TestResults {
                    name: "test3/0".to_string(),
                    status: SubmitStatus::Ok,
                },
            ]),
        };

        actual.print_with_tests();
        assert_eq!(actual, expected);
    }

    #[test]
    fn incorrect_17_mn_parse_test() {
        let baca = ConnectionConfig {
            host: "mn".to_string(),
            login: "".to_string(),
            password: "".to_string(),
            permutation: "permutation".to_string(),
            cookie: "cookie".to_string(),
        };
        let raw = r#"//OK[0,9,53,2,5,36,52,2,5,36,51,2,5,36,50,2,5,36,49,2,5,36,48,2,5,7,47,2,5,7,46,2,5,7,45,2,5,8,4,3,0,44,43,42,41,40,39,38,37,8,5,36,35,34,33,32,31,30,29,8,5,1,4,3,28,0,27,26,25,24,23,22,21,20,19,9,5,18,17,16,15,14,13,12,11,10,9,5,1,4,3,0,0,9,8,2,5,7,6,2,5,1,4,3,2,1,["testerka.gwt.client.submits.SubmitDetailsModel/2564112456","compilation logs with status and test strings","testerka.gwt.client.tools.DataSource/1474249525","[[Ljava.lang.String;/4182515373","[Ljava.lang.String;/2600011424","320","program zaakceptowany","czas","status","[E] Metoda SOR","4","2020-04-23 09:19:09","2020-05-11 23:00:00","2020-05-25 23:00:00","122","30","1024","125","Nazwa zdania","Liczba punktow do zdobycia","Start zadania","Termin oddania","Koniec zdania","Limit pamieci (MB)","Limit czasu kompilacji (s)","Limit pamieci na kompilacje (MB)","Limit kodu zrodlowego (kB)","code","3266","C++","2020-04-26 12:43:36","1970","17","100","0.67","przekroczony czas","id","język","czas zgłoszenia","rozmiar (b)","zaliczone (%)","spoznienie (%)","punkty","nazwa statusu","testy/test1","testy/test2","testy/test3","testy/test4","testy/test5","testy/test6","testy/test7","testy/test8","test"],0,7]"#;

        let actual = Submit::parse(&baca, raw);
        let expected = Submit {
            status: SubmitStatus::TimeExceeded,
            points: 0.67,
            lateness: Some(100),
            accepted: 17,
            size: 1970,
            timestamp: "2020-04-26 12:43:36".to_string(),
            language: "C++".to_string(),
            id: "3266".to_string(),
            max_points: Some(4),
            problem_name: "[E] Metoda SOR".to_string(),
            link: "https://baca.ii.uj.edu.pl/mn/#SubmitDetails/3266".to_string(),
            test_results: Some(vec![
                TestResults {
                    name: "testy/test1".to_string(),
                    status: SubmitStatus::Ok,
                },
                TestResults {
                    name: "testy/test2".to_string(),
                    status: SubmitStatus::Ok,
                },
                TestResults {
                    name: "testy/test3".to_string(),
                    status: SubmitStatus::Ok,
                },
                TestResults {
                    name: "testy/test4".to_string(),
                    status: SubmitStatus::TimeExceeded,
                },
                TestResults {
                    name: "testy/test5".to_string(),
                    status: SubmitStatus::TimeExceeded,
                },
                TestResults {
                    name: "testy/test6".to_string(),
                    status: SubmitStatus::TimeExceeded,
                },
                TestResults {
                    name: "testy/test7".to_string(),
                    status: SubmitStatus::TimeExceeded,
                },
                TestResults {
                    name: "testy/test8".to_string(),
                    status: SubmitStatus::TimeExceeded,
                },
            ]),
        };

        actual.print_with_tests();
        assert_eq!(actual, expected);
    }

    #[test]
    fn incorrect_80_mp_parse_test() {
        let baca = ConnectionConfig {
            host: "mp".to_string(),
            login: "".to_string(),
            password: "".to_string(),
            permutation: "permutation".to_string(),
            cookie: "cookie".to_string(),
        };
        let raw = r#"//OK[0,9,54,2,5,7,53,2,5,46,52,2,5,7,51,2,5,46,50,2,5,7,49,2,5,46,48,2,5,36,47,2,5,46,45,2,5,8,4,3,0,44,43,42,41,40,39,38,37,8,5,36,35,34,33,32,31,30,29,8,5,1,4,3,28,0,27,26,25,24,23,22,21,20,19,9,5,18,17,16,15,14,13,12,11,10,9,5,1,4,3,0,0,9,8,2,5,7,6,2,5,1,4,3,2,1,["testerka.gwt.client.submits.SubmitDetailsModel/2564112456","logs","testerka.gwt.client.tools.DataSource/1474249525","[[Ljava.lang.String;/4182515373","[Ljava.lang.String;/2600011424","496","program zaakceptowany","czas","status","P05","3","2019-04-11 12:00:24","2019-04-25 22:00:24","2019-05-02 22:00:24","122","30","1024","125","Nazwa zdania","Liczba punktow do zdobycia","Start zadania","Termin oddania","Koniec zdania","Limit pamieci (MB)","Limit czasu kompilacji (s)","Limit pamieci na kompilacje (MB)","Limit kodu zrodlowego (kB)","code","2484","Java","2019-04-12 23:54:34","1944","38","100","1.13","przekroczony czas","id","język","czas zgłoszenia","rozmiar (b)","zaliczone (%)","spoznienie (%)","punkty","nazwa statusu","test_nsum/big_nums_iter","zĹ\x82a odpowiedz","test_nsum/big_nums_rec","test_nsum/big_powers_iter","test_nsum/big_powers_rec","test_nsum/jawny_test_iter","test_nsum/jawny_test_rec","test_nsum/simple_iter","test_nsum/simple_rec","test"],0,7]"#;

        let actual = Submit::parse(&baca, raw);
        let expected = Submit {
            status: SubmitStatus::TimeExceeded,
            points: 1.13,
            lateness: Some(100),
            accepted: 38,
            size: 1944,
            timestamp: "2019-04-12 23:54:34".to_string(),
            language: "Java".to_string(),
            id: "2484".to_string(),
            max_points: Some(3),
            problem_name: "P05".to_string(),
            link: "https://baca.ii.uj.edu.pl/mp/#SubmitDetails/2484".to_string(),
            test_results: Some(vec![
                TestResults {
                    name: "test_nsum/big_nums_iter".to_string(),
                    status: SubmitStatus::WrongAnswer,
                },
                TestResults {
                    name: "test_nsum/big_nums_rec".to_string(),
                    status: SubmitStatus::TimeExceeded,
                },
                TestResults {
                    name: "test_nsum/big_powers_iter".to_string(),
                    status: SubmitStatus::WrongAnswer,
                },
                TestResults {
                    name: "test_nsum/big_powers_rec".to_string(),
                    status: SubmitStatus::Ok,
                },
                TestResults {
                    name: "test_nsum/jawny_test_iter".to_string(),
                    status: SubmitStatus::WrongAnswer,
                },
                TestResults {
                    name: "test_nsum/jawny_test_rec".to_string(),
                    status: SubmitStatus::Ok,
                },
                TestResults {
                    name: "test_nsum/simple_iter".to_string(),
                    status: SubmitStatus::WrongAnswer,
                },
                TestResults {
                    name: "test_nsum/simple_rec".to_string(),
                    status: SubmitStatus::Ok,
                },
            ]),
        };

        actual.print_with_tests();
        assert_eq!(actual, expected);
    }

    #[test]
    fn correct_pn_parse_test() {
        let baca = ConnectionConfig {
            host: "pn".to_string(),
            login: "".to_string(),
            password: "".to_string(),
            permutation: "permutation".to_string(),
            cookie: "cookie".to_string(),
        };
        let raw = r#"//OK[0,12,72,71,70,4,5,7,69,58,68,4,5,7,59,67,66,4,5,7,65,63,64,4,5,7,59,63,62,4,5,7,61,58,60,4,5,7,59,58,57,4,5,6,4,3,0,56,55,54,53,52,51,50,49,8,5,7,48,47,47,46,45,44,43,8,5,1,4,3,42,0,41,40,39,38,37,36,35,34,33,9,5,32,31,30,29,28,28,27,26,25,9,5,1,4,3,0,24,23,22,21,20,19,6,5,18,17,16,15,14,13,6,5,1,4,3,0,12,11,2,5,7,6,2,5,7,10,2,5,7,9,2,5,7,8,2,5,7,8,2,5,7,6,2,5,6,4,3,2,1,["testerka.gwt.client.submits.SubmitDetailsModel/2564112456","logs","testerka.gwt.client.tools.DataSource/1474249525","[[Ljava.lang.String;/4182515373","[Ljava.lang.String;/2600011424","60","program zaakceptowany","56","64","68","czas","status","nazwisko","Imie","Nazwisko","nick","grupa nr 1","Prowadzacy","login","imię","nazwisko","nick","grupa","prowadzący","OPT1: MinMax","2","2020-12-16 09:58:00","2021-01-24 23:30:00","122","30","1024","125","Nazwa zdania","Liczba punktow do zdobycia","Start zadania","Termin oddania","Koniec zdania","Limit pamieci (MB)","Limit czasu kompilacji (s)","Limit pamieci na kompilacje (MB)","Limit kodu zrodlowego (kB)","code","478","C++","2021-01-13 12:27:10","991","100","2.00","id","język","czas zgłoszenia","rozmiar (b)","zaliczone (%)","spoznienie (%)","punkty","nazwa statusu","0/0","768","10000","0_t/0","1200","1/0","924","1_t/0","1050","2/0","772","2_t/0","1000","test","time","limit czasu"],0,7]"#;

        let actual = Submit::parse(&baca, raw);
        let expected = Submit {
            status: SubmitStatus::Ok,
            points: 2.0,
            lateness: Some(100),
            accepted: 100,
            size: 991,
            timestamp: "2021-01-13 12:27:10".to_string(),
            language: "C++".to_string(),
            id: "478".to_string(),
            max_points: Some(2),
            problem_name: "OPT1: MinMax".to_string(),
            link: "https://baca.ii.uj.edu.pl/pn/#SubmitDetails/478".to_string(),
            test_results: Some(vec![
                TestResults {
                    name: "0/0".to_string(),
                    status: SubmitStatus::Ok,
                },
                TestResults {
                    name: "0_t/0".to_string(),
                    status: SubmitStatus::Ok,
                },
                TestResults {
                    name: "1/0".to_string(),
                    status: SubmitStatus::Ok,
                },
                TestResults {
                    name: "1_t/0".to_string(),
                    status: SubmitStatus::Ok,
                },
                TestResults {
                    name: "2/0".to_string(),
                    status: SubmitStatus::Ok,
                },
                TestResults {
                    name: "2_t/0".to_string(),
                    status: SubmitStatus::Ok,
                },
            ]),
        };

        actual.print_with_tests();
        assert_eq!(actual, expected);
    }

    #[test]
    fn no_tests_mn_parse_test() {
        let baca = ConnectionConfig {
            host: "pn".to_string(),
            login: "".to_string(),
            password: "".to_string(),
            permutation: "permutation".to_string(),
            cookie: "cookie".to_string(),
        };
        let raw = r#"//OK[0,9,45,2,5,0,4,3,0,44,43,42,41,40,39,38,37,8,5,36,35,34,33,32,31,30,29,8,5,1,4,3,28,0,27,26,25,24,23,22,21,20,19,9,5,18,17,16,15,14,13,12,11,10,9,5,1,4,3,0,0,9,8,2,5,7,6,2,5,1,4,3,2,1,["testerka.gwt.client.submits.SubmitDetailsModel/2564112456","\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\nLogi kolejnej kompilacji:\n\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\nmake: Entering directory `/var/lib/baca/work\x27\ng++ -O2 source.cpp -o out -static -m32\nsource.cpp:7:7: error: expected nested-name-specifier before \x27vec\x27\nsource.cpp:7:7: error: \x27vec\x27 has not been declared\nsource.cpp:7:11: error: expected \x27;\x27 before \x27\x3D\x27 token\nsource.cpp:7:11: error: expected unqualified-id before \x27\x3D\x27 token\nsource.cpp:14:11: error: expected nested-name-specifier before \x27value_type\x27\nsource.cpp:14:11: error: using-declaration for non-member at class scope\nsource.cpp:14:22: error: expected \x27;\x27 before \x27\x3D\x27 token\nsource.cpp:14:22: error: expected unqualified-id before \x27\x3D\x27 token\nsource.cpp:15:11: error: expected nested-name-specifier before \x27container_type\x27\nsource.cpp:15:11: error: using-declaration for non-member at class scope\nsource.cpp:15:26: error: expected \x27;\x27 before \x27\x3D\x27 token\nsource.cpp:15:26: error: expected unqualified-id before \x27\x3D\x27 token\nsource.cpp:38:9: error: \x27value_type\x27 does not name a type\nsource.cpp:39:9: error: \x27value_type\x27 does not name a type\nsource.cpp:43:11: error: expected nested-name-specifier before \x27map_type\x27\nsource.cpp:43:11: error: using-declaration for non-member at class scope\nsource.cpp:43:20: error: expected \x27;\x27 before \x27\x3D\x27 token\nsource.cpp:43:20: error: expected unqualified-id before \x27\x3D\x27 token\nsource.cpp:45:31: error: \x27container_type\x27 does not name a type\nsource.cpp:45:47: error: ISO C++ forbids declaration of \x27nodes\x27 with no type [-fpermissive]\nsource.cpp:46:31: error: \x27container_type\x27 does not name a type\nsource.cpp:46:47: error: ISO C++ forbids declaration of \x27values\x27 with no type [-fpermissive]\nsource.cpp:67:5: error: \x27container_type\x27 does not name a type\nsource.cpp:77:5: error: \x27value_type\x27 does not name a type\nsource.cpp:83:5: error: \x27value_type\x27 does not name a type\nsource.cpp:137:5: error: \x27container_type\x27 does not name a type\nsource.cpp:138:5: error: \x27map_type\x27 does not name a type\nsource.cpp: In function \x27bool mn::operator\x3C(const mn::hermite_polynomial::slice\x3CIter\x3E\x26, const mn::hermite_polynomial::slice\x3CIter\x3E\x26)\x27:\nsource.cpp:26:20: error: \x27tie\x27 is not a member of \x27std\x27\nsource.cpp:26:49: error: \x27tie\x27 is not a member of \x27std\x27\nsource.cpp: In member function \x27mn::hermite_polynomial::slice\x3CIter\x3E mn::hermite_polynomial::make_slice(Iter, Iter)\x27:\nsource.cpp:33:16: warning: extended initializer lists only available with -std\x3Dc++11 or -std\x3Dgnu++11 [enabled by default]\nsource.cpp: In constructor \x27mn::hermite_polynomial::hermite_polynomial(const int\x26, const int\x26)\x27:\nsource.cpp:48:31: error: request for member \x27begin\x27 in \x27nodes\x27, which is of non-class type \x27const int\x27\nsource.cpp:49:31: error: request for member \x27end\x27 in \x27nodes\x27, which is of non-class type \x27const int\x27\nsource.cpp:50:32: error: request for member \x27begin\x27 in \x27values\x27, which is of non-class type \x27const int\x27\nsource.cpp:52:29: error: \x27value_type\x27 has not been declared\nsource.cpp:52:43: error: \x27value_type\x27 has not been declared\nsource.cpp: In lambda function:\nsource.cpp:53:36: warning: extended initializer lists only available with -std\x3Dc++11 or -std\x3Dgnu++11 [enabled by default]\nsource.cpp:53:48: error: too many initializers for \x27mn::hermite_polynomial::pair\x27\nsource.cpp: In constructor \x27mn::hermite_polynomial::hermite_polynomial(const int\x26, const int\x26)\x27:\nsource.cpp:54:25: warning: lambda expressions only available with -std\x3Dc++11 or -std\x3Dgnu++11 [enabled by default]\nsource.cpp:56:14: error: \x27yit\x27 does not name a type\nsource.cpp:57:20: error: \x27it\x27 does not name a type\nsource.cpp:57:43: error: expected \x27;\x27 before \x27it\x27\nsource.cpp:57:43: error: \x27it\x27 was not declared in this scope\nsource.cpp:58:23: error: \x27yit\x27 was not declared in this scope\nsource.cpp:62:17: error: \x27quotients\x27 was not declared in this scope\nsource.cpp: In member function \x27void mn::hermite_polynomial::interpolate()\x27:\nsource.cpp:103:13: error: \x27coeffs_\x27 was not declared in this scope\nsource.cpp: In member function \x27double mn::hermite_polynomial::get_quotient(mn::hermite_polynomial::slice\x3CIter\x3E)\x27:\nsource.cpp:111:14: error: \x27quotients\x27 was not declared in this scope\nsource.cpp:118:18: error: \x27k\x27 does not name a type\nsource.cpp:119:18: error: \x27i\x27 does not name a type\nsource.cpp:123:78: error: expected primary-expression before \x27)\x27 token\nsource.cpp:123:78: error: expected \x27;\x27 before \x27)\x27 token\nsource.cpp:124:13: error: \x27quotients\x27 was not declared in this scope\nsource.cpp:124:32: error: \x27i\x27 was not declared in this scope\nsource.cpp:124:36: error: \x27k\x27 was not declared in this scope\nsource.cpp:128:14: error: \x27f1\x27 does not name a type\nsource.cpp:129:14: error: \x27f2\x27 does not name a type\nsource.cpp:130:14: error: \x27q\x27 does not name a type\nsource.cpp:131:9: error: \x27quotients\x27 was not declared in this scope\nsource.cpp:131:26: error: \x27q\x27 was not declared in this scope\nsource.cpp: At global scope:\nsource.cpp:141:1: error: \x27vec\x27 does not name a type\nsource.cpp:152:31: error: \x27vec\x27 was not declared in this scope\nsource.cpp:152:34: error: template argument 2 is invalid\nsource.cpp: In function \x27int mn::read_data(std::istream\x26)\x27:\nsource.cpp:154:10: error: \x27node_count\x27 does not name a type\nsource.cpp:155:10: error: \x27point_count\x27 does not name a type\nsource.cpp:157:11: error: \x27node_count\x27 was not declared in this scope\nsource.cpp:157:25: error: \x27point_count\x27 was not declared in this scope\nsource.cpp:158:10: error: \x27nodes\x27 does not name a type\nsource.cpp:159:10: error: \x27values\x27 does not name a type\nsource.cpp:160:10: error: \x27points\x27 does not name a type\nsource.cpp:162:48: error: \x27nodes\x27 was not declared in this scope\nsource.cpp:162:55: error: \x27values\x27 was not declared in this scope\nsource.cpp:162:65: error: \x27points\x27 was not declared in this scope\nsource.cpp: At global scope:\nsource.cpp:166:39: error: ISO C++ forbids declaration of \x27print_container\x27 with no type [-fpermissive]\nsource.cpp:166:39: error: top-level declaration of \x27print_container\x27 specifies \x27auto\x27\nsource.cpp:166:39: error: trailing return type only available with -std\x3Dc++11 or -std\x3Dgnu++11\nsource.cpp: In function \x27int main()\x27:\nsource.cpp:182:10: error: \x27data\x27 does not name a type\nsource.cpp:183:11: error: ISO C++ forbids declaration of \x27polynomial\x27 with no type [-fpermissive]\nsource.cpp:183:24: error: \x27data\x27 was not declared in this scope\nsource.cpp:184:11: error: ISO C++ forbids declaration of \x27points\x27 with no type [-fpermissive]\nsource.cpp:185:5: error: \x27print_container\x27 is not a member of \x27mn\x27\nsource.cpp:185:37: error: request for member \x27coefficients\x27 in \x27polynomial\x27, which is of non-class type \x27int\x27\nsource.cpp:187:23: error: ISO C++ forbids declaration of \x27point\x27 with no type [-fpermissive]\nsource.cpp:187:31: error: range-based \x27for\x27 loops are not allowed in C++98 mode\nsource.cpp:189:33: error: request for member \x27at\x27 in \x27polynomial\x27, which is of non-class type \x27int\x27\nsource.cpp: In instantiation of \x27double mn::hermite_polynomial::get_quotient(mn::hermite_polynomial::slice\x3CIter\x3E) [with Iter \x3D __gnu_cxx::__normal_iterator\x3Cmn::hermite_polynomial::pair*, std::vector\x3Cmn::hermite_polynomial::pair\x3E \x3E]\x27:\nsource.cpp:104:66:   required from here\nsource.cpp:116:9: error: \x27struct mn::hermite_polynomial::pair\x27 has no member named \x27x\x27\nsource.cpp:116:9: error: \x27struct mn::hermite_polynomial::pair\x27 has no member named \x27x\x27\nmake: *** [1] Error 1\nmake: Leaving directory `/var/lib/baca/work\x27\n","testerka.gwt.client.tools.DataSource/1474249525","[[Ljava.lang.String;/4182515373","[Ljava.lang.String;/2600011424","136","bĹ\x82Ä\x85d wykonania","czas","status","[F] Interpolacja","4","2020-04-24 11:21:54","2020-05-28 23:00:00","2020-06-11 23:00:00","122","30","1024","125","Nazwa zdania","Liczba punktow do zdobycia","Start zadania","Termin oddania","Koniec zdania","Limit pamieci (MB)","Limit czasu kompilacji (s)","Limit pamieci na kompilacje (MB)","Limit kodu zrodlowego (kB)","source code with test and status","4070","C++","2020-05-14 13:11:52","4381","0","100","0.00","bĹ\x82Ä\x85d kompilacji","id","język","czas zgłoszenia","rozmiar (b)","zaliczone (%)","spoznienie (%)","punkty","nazwa statusu","test"],0,7]"#;

        let actual = Submit::parse(&baca, raw);
        let expected = Submit {
            status: SubmitStatus::CompileError,
            points: 0.0,
            lateness: Some(100),
            accepted: 0,
            size: 4381,
            timestamp: "2020-05-14 13:11:52".to_string(),
            language: "C++".to_string(),
            id: "4070".to_string(),
            max_points: Some(4),
            problem_name: "[F] Interpolacja".to_string(),
            link: "https://baca.ii.uj.edu.pl/pn/#SubmitDetails/4070".to_string(),
            test_results: None,
        };

        actual.print_with_tests();
        assert_eq!(actual, expected);
    }

    #[test]
    fn one_testcase_no_file_provided() {
        let baca = ConnectionConfig {
            host: "so2018".to_string(),
            login: "".to_string(),
            password: "".to_string(),
            permutation: "permutation".to_string(),
            cookie: "cookie".to_string(),
        };
        let raw = r#"//OK[0,9,43,2,5,7,42,2,5,1,4,3,0,41,40,39,38,37,36,35,34,8,5,7,33,32,6,6,31,30,29,8,5,1,4,3,28,0,27,26,25,24,23,22,21,20,19,9,5,18,17,16,15,14,13,12,11,10,9,5,1,4,3,0,0,9,8,2,5,7,6,2,5,1,4,3,2,1,["testerka.gwt.client.submits.SubmitDetailsModel/2564112456","brak pliku","testerka.gwt.client.tools.DataSource/1474249525","[[Ljava.lang.String;/4182515373","[Ljava.lang.String;/2600011424","0","brak nagĹ\x82Ăłwka","czas","status","Kupcy i piraci","7","2018-06-06 08:00:00","2018-06-21 21:00:00","2018-06-28 21:00:00","122","30","1024","125","Nazwa zdania","Liczba punktow do zdobycia","Start zadania","Termin oddania","Koniec zdania","Limit pamieci (MB)","Limit czasu kompilacji (s)","Limit pamieci na kompilacje (MB)","Limit kodu zrodlowego (kB)","","1964","Java","2018-06-20 12:50:51","100","0.00","id","język","czas zgłoszenia","rozmiar (b)","zaliczone (%)","spoznienie (%)","punkty","nazwa statusu","1/test","test"],0,7]"#;
        let actual = Submit::parse(&baca, raw);
        let expected = Submit {
            status: SubmitStatus::NoHeader,
            points: 0.0,
            lateness: Some(100),
            accepted: 0,
            size: 0,
            timestamp: "2018-06-20 12:50:51".to_string(),
            language: "Java".to_string(),
            id: "1964".to_string(),
            max_points: Some(7),
            problem_name: "Kupcy i piraci".to_string(),
            link: "https://baca.ii.uj.edu.pl/so2018/#SubmitDetails/1964".to_string(),
            test_results: Some(vec![TestResults {
                name: "1/test".to_string(),
                status: SubmitStatus::NoHeader,
            }]),
        };

        actual.print_with_tests();
        assert_eq!(actual, expected);
    }

    #[test]
    fn two_testcases() {
        let baca = ConnectionConfig {
            host: "p1".to_string(),
            login: "".to_string(),
            password: "".to_string(),
            permutation: "permutation".to_string(),
            cookie: "cookie".to_string(),
        };
        let raw = r#"//OK[0,9,45,2,5,7,44,2,5,7,43,2,5,2,4,3,0,42,41,40,39,38,37,36,35,8,5,7,34,33,33,32,31,30,29,8,5,1,4,3,28,0,27,26,25,24,23,22,21,20,19,9,5,18,17,16,15,14,13,12,11,10,9,5,1,4,3,0,0,9,8,2,5,7,6,2,5,1,4,3,2,1,["testerka.gwt.client.submits.SubmitDetailsModel/2564112456","","testerka.gwt.client.tools.DataSource/1474249525","[[Ljava.lang.String;/4182515373","[Ljava.lang.String;/2600011424","184","program zaakceptowany","czas","status","Treningowe 2","0","2018-10-25 17:00:00","2018-11-01 17:00:00","2018-11-08 17:00:00","122","30","1024","125","Nazwa zdania","Liczba punktow do zdobycia","Start zadania","Termin oddania","Koniec zdania","Limit pamieci (MB)","Limit czasu kompilacji (s)","Limit pamieci na kompilacje (MB)","Limit kodu zrodlowego (kB)","//Hubert Jaremko\r\n#include \x3Ciostream\x3E\r\n\r\nusing namespace std;\r\n\r\nint main()\r\n{\r\n    int dataAmount \x3D 0;\r\n    int id \x3D 0;\r\n    int labPercent \x3D 0;\r\n    int bacaPercent \x3D 0;\r\n    int sum \x3D 0;\r\n\r\n    cin \x3E\x3E dataAmount;\r\n\r\n    while ( dataAmount-- )\r\n    {\r\n        cin \x3E\x3E id \x3E\x3E labPercent \x3E\x3E bacaPercent;\r\n        sum \x3D labPercent + bacaPercent;\r\n\r\n        cout \x3C\x3C id \x3C\x3C \x27 \x27 \x3C\x3C sum \x3C\x3C \"% \";\r\n\r\n        if( sum \x3E\x3D 90 )\r\n            cout \x3C\x3C \"bardzo dobry (5.0)\";\r\n        else if( sum \x3E\x3D 80 )\r\n            cout \x3C\x3C \"dobry plus (4.5)\";\r\n        else if( sum \x3E\x3D 70 )\r\n            cout \x3C\x3C \"dobry (4.0)\";\r\n        else if( sum \x3E\x3D 60 )\r\n            cout \x3C\x3C \"dostateczny plus (3.5)\";\r\n        else if( sum \x3E\x3D 50 )\r\n            cout \x3C\x3C \"dostateczny (3.0)\";\r\n        else\r\n            cout \x3C\x3C \"niedostateczny (2.0)\";\r\n\r\n        cout \x3C\x3C endl;\r\n    }\r\n\r\n    return 0;\r\n}\r\n","57","C++","2018-10-26 00:55:00","856","100","0.00","id","język","czas zgłoszenia","rozmiar (b)","zaliczone (%)","spoznienie (%)","punkty","nazwa statusu","test_0","test_1","test"],0,7]"#;
        let actual = Submit::parse(&baca, raw);
        let expected = Submit {
            status: SubmitStatus::Ok,
            points: 0.0,
            lateness: Some(100),
            accepted: 100,
            size: 856,
            timestamp: "2018-10-26 00:55:00".to_string(),
            language: "C++".to_string(),
            id: "57".to_string(),
            max_points: Some(0),
            problem_name: "Treningowe 2".to_string(),
            link: "https://baca.ii.uj.edu.pl/p1/#SubmitDetails/57".to_string(),
            test_results: Some(vec![
                TestResults {
                    name: "test_0".to_string(),
                    status: SubmitStatus::Ok,
                },
                TestResults {
                    name: "test_1".to_string(),
                    status: SubmitStatus::Ok,
                },
            ]),
        };

        actual.print_with_tests();
        assert_eq!(actual, expected);
    }

    #[test]
    fn real_submit_2888() {
        let baca = ConnectionConfig {
            host: "p1".to_string(),
            login: "".to_string(),
            password: "".to_string(),
            permutation: "permutation".to_string(),
            cookie: "cookie".to_string(),
        };
        let raw = r#"//OK[0,10,65,2,5,7,64,2,5,7,63,2,5,7,62,2,5,7,61,2,5,7,60,2,5,7,59,2,5,7,58,2,5,7,57,2,5,37,56,2,5,37,55,2,5,7,54,2,5,7,53,2,5,7,52,2,5,7,51,2,5,7,50,2,5,7,49,2,5,7,48,2,5,7,47,2,5,7,46,2,5,19,4,3,0,45,44,43,42,41,40,39,38,8,5,37,36,35,34,33,32,31,30,8,5,1,4,3,29,0,28,27,26,25,24,23,22,21,20,9,5,19,18,17,16,15,14,13,12,11,9,5,1,4,3,0,0,10,9,2,5,7,8,2,5,7,6,2,5,2,4,3,2,1,["testerka.gwt.client.submits.SubmitDetailsModel/2564112456","\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\nLogi kolejnej kompilacji:\n\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\nmake: Entering directory `/var/lib/baca/work\x27\ng++ main.cpp source.cpp -std\x3Dc++11 -O2 -o out -static -m32\nmake: Leaving directory `/var/lib/baca/work\x27\n\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\nLogi kolejnej kompilacji:\n\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\x3D\nmake: Entering directory `/var/lib/baca/work\x27\ng++ main.cpp source.cpp -std\x3Dc++11 -O2 -o out -static -m32\nmake: Leaving directory `/var/lib/baca/work\x27\n","testerka.gwt.client.tools.DataSource/1474249525","[[Ljava.lang.String;/4182515373","[Ljava.lang.String;/2600011424","748","program zaakceptowany","720","czas","status","[D] Skalowany Gauss","4","2020-04-15 15:30:32","2020-04-30 23:00:00","2020-05-15 23:00:00","122","30","1024","125","Nazwa zdania","Liczba punktow do zdobycia","Start zadania","Termin oddania","Koniec zdania","Limit pamieci (MB)","Limit czasu kompilacji (s)","Limit pamieci na kompilacje (MB)","Limit kodu zrodlowego (kB)","// Hubert Jaremko\r\n#define NDEBUG\r\n#include \"vectalg.h\"\r\n#include \x3Calgorithm\x3E\r\n#include \x3Cnumeric\x3E\r\n\r\nVector operator+( const Vector\x26 lhs, const Vector\x26 rhs )\r\n{\r\n    auto result \x3D Vector( lhs.size() );\r\n    std::transform( lhs.begin(),\r\n                    lhs.end(),\r\n                    rhs.begin(),\r\n                    result.begin(),\r\n                    std::plus\x3Cdouble\x3E() );\r\n    return result;\r\n}\r\n\r\nclass equation_solver\r\n{\r\npublic:\r\n    equation_solver( const Matrix\x26 a_0, const Vector\x26 b_0, double eps_ )\r\n        : A0( a_0 ), b0( b_0 ), eps( eps_ ), perm_vec( A0.size() ),\r\n          row_norms( A0.size() )\r\n    {\r\n        std::iota( perm_vec.begin(), perm_vec.end(), 0 );\r\n    }\r\n\r\n    Vector solve()\r\n    {\r\n        calculate_lu();\r\n        auto sol \x3D Vector(A.size());\r\n        solve_system_upper( bx, sol );\r\n        return permute( fix_solution( sol ) );\r\n//        return permute( fix_solution( solve_system_upper( bx ) ) );\r\n        //        return fix_solution( solve_system_upper( bx ) );\r\n    }\r\n\r\nprivate:\r\n    Vector permute( const Vector\x26 v )\r\n    {\r\n        auto result \x3D Vector( v.size() );\r\n\r\n        for ( int i \x3D 0; i \x3C v.size(); ++i )\r\n        {\r\n            result[ i ] \x3D v[ perm_vec[ i ] ];\r\n        }\r\n\r\n        return result;\r\n    }\r\n\r\n    double\x26 at( size_t i, size_t j )\r\n    {\r\n        return A( perm_vec[ i ], j );\r\n    }\r\n\r\n    double at( size_t i, size_t j ) const\r\n    {\r\n        return A( perm_vec[ i ], j );\r\n    }\r\n\r\n    double\x26 arr_at( Vector\x26 cont, size_t i )\r\n    {\r\n        return cont[ perm_vec[ i ] ];\r\n    }\r\n\r\n    double arr_at( const Vector\x26 cont, size_t i ) const\r\n    {\r\n        return cont[ perm_vec[ i ] ];\r\n    }\r\n\r\n    void calculate_row_norms()\r\n    {\r\n        for ( int i \x3D 0; i \x3C A.size(); ++i )\r\n        {\r\n            auto norm \x3D 0.0;\r\n            for ( int j \x3D 0; j \x3C A.size(); ++j )\r\n            {\r\n                norm \x3D std::max( norm, std::abs( at( i, j ) ) );\r\n            }\r\n            arr_at( row_norms, i ) \x3D norm;\r\n        }\r\n    }\r\n\r\n    size_t pick_main_element( size_t k ) const\r\n    {\r\n        auto max_elem \x3D k;\r\n\r\n        for ( auto i \x3D k; i \x3C A.size(); ++i )\r\n        {\r\n            const auto scale \x3D std::abs( at( i, k ) ) / arr_at( row_norms, i );\r\n            if ( scale \x3E\r\n                 std::abs( at( max_elem, k ) ) / arr_at( row_norms, max_elem ) )\r\n            {\r\n                max_elem \x3D i;\r\n            }\r\n        }\r\n\r\n        return max_elem;\r\n    }\r\n\r\n    void eliminate( size_t k )\r\n    {\r\n        for ( auto i \x3D k + 1; i \x3C A.size(); ++i )\r\n        {\r\n            const auto multiplier \x3D at( i, k ) / at( k, k );\r\n            at( i, k ) \x3D multiplier;\r\n\r\n            for ( auto j \x3D k + 1; j \x3C A.size(); ++j )\r\n            {\r\n                at( i, j ) -\x3D multiplier * at( k, j );\r\n            }\r\n\r\n            arr_at( bx, i ) -\x3D multiplier * arr_at( bx, k );\r\n        }\r\n    }\r\n\r\n    void calculate_lu()\r\n    {\r\n        calculate_row_norms();\r\n\r\n        for ( auto k \x3D 0; k \x3C A.size() - 1; ++k )\r\n        {\r\n            auto p \x3D pick_main_element( k );\r\n            std::swap( perm_vec[ p ], perm_vec[ k ] );\r\n            eliminate( k );\r\n        }\r\n    }\r\n\r\n//    Vector solve_system_upper( const Vector\x26 b )\r\n    void solve_system_upper( const Vector\x26 b, Vector\x26 result )\r\n    {\r\n        const auto n \x3D A.size();\r\n//        Vector result( n );\r\n\r\n        for ( int i \x3D n - 1; i \x3E\x3D 0; --i )\r\n        {\r\n            auto sum \x3D 0.0;\r\n            for ( auto j \x3D i + 1; j \x3C n; ++j )\r\n            {\r\n                sum +\x3D at( i, j ) * arr_at( result, j );\r\n            }\r\n\r\n            arr_at( result, i ) \x3D ( arr_at( b, i ) - sum ) / at( i, i );\r\n        }\r\n\r\n//        return result;\r\n    }\r\n\r\n//    Vector solve_system_lower( const Vector\x26 b )\r\n    void solve_system_lower( const Vector\x26 b, Vector\x26 result )\r\n    {\r\n        const auto n \x3D A.size();\r\n//        Vector result( n );\r\n\r\n        for ( int i \x3D 0; i \x3C n; ++i )\r\n        {\r\n            auto sum \x3D 0.0;\r\n            for ( auto j \x3D 0; j \x3C i; ++j )\r\n            {\r\n                sum +\x3D at( i, j ) * arr_at( result, j );\r\n            }\r\n\r\n            arr_at( result, i ) \x3D arr_at( b, i ) - sum;\r\n        }\r\n\r\n//        return result;\r\n    }\r\n\r\n    //    Vector residual_vector( const Vector\x26 x ) const\r\n    void residual_vector( const Vector\x26 x, Vector\x26 result ) const\r\n    {\r\n        const auto n \x3D b0.size();\r\n        //        Vector result( n );\r\n        for ( int i \x3D 0; i \x3C n; ++i )\r\n        {\r\n            long double s \x3D 0.0;\r\n            for ( int j \x3D 0; j \x3C n; ++j )\r\n            {\r\n                s +\x3D static_cast\x3Clong double\x3E( A0( i, j ) ) * arr_at( x, j );\r\n            }\r\n            long double bb \x3D b0[ i ];\r\n            result[ i ] \x3D static_cast\x3Cdouble\x3E( bb - s );\r\n        }\r\n//        return result;\r\n    }\r\n\r\n    Vector fix_solution( Vector x0 )\r\n    {\r\n        //        auto residual \x3D residual_vector( x0 );\r\n        auto residual \x3D Vector( A.size() );\r\n        auto result \x3D Vector( A.size() );\r\n        residual_vector( x0, residual );\r\n\r\n        while ( residual.max_norm() \x3E\x3D eps )\r\n        {\r\n            solve_system_lower( residual,result );\r\n            solve_system_upper( result,result );\r\n            //            x0 \x3D x0 + solve_system_upper( solve_system_lower( residual ) );\r\n            //            residual \x3D residual_vector( x0 );\r\n            x0 \x3D x0 + result;\r\n            residual_vector( x0, residual );\r\n        }\r\n\r\n        return x0;\r\n    }\r\n\r\n    const Matrix\x26 A0;\r\n    const Vector\x26 b0;\r\n    Vector bx { b0 };\r\n    Matrix A { A0 };\r\n    double eps { 0.0 };\r\n    Vector perm_vec;\r\n    Vector row_norms;\r\n};\r\n\r\nVector solveEquations( const Matrix\x26 A0, const Vector\x26 b0, double eps )\r\n{\r\n    return equation_solver( A0, b0, eps ).solve();\r\n}\r\n","2888","C++","2020-04-22 13:04:36","5752","89","100","3.58","przekroczony czas","id","język","czas zgłoszenia","rozmiar (b)","zaliczone (%)","spoznienie (%)","punkty","nazwa statusu","testy_jawne/test1","testy_jawne/test2","testy_jawne/test3","testy_jawne/test4","testy_jawne/test5","testy_jawne/test6","testy_jawne/test8","testy/test0","testy/test1","testy/test10","testy/test11","testy/test2","testy/test3","testy/test4","testy/test5","testy/test6","testy/test7","testy/test8","testy/test9","test"],0,7]"#;
        let actual = Submit::parse(&baca, raw);
        let expected = Submit {
            status: SubmitStatus::TimeExceeded,
            points: 3.58,
            lateness: Some(100),
            accepted: 89,
            size: 5752,
            timestamp: "2020-04-22 13:04:36".to_string(),
            language: "C++".to_string(),
            id: "2888".to_string(),
            max_points: Some(4),
            problem_name: "[D] Skalowany Gauss".to_string(),
            link: "https://baca.ii.uj.edu.pl/p1/#SubmitDetails/2888".to_string(),
            test_results: Some(vec![
                TestResults {
                    name: "testy_jawne/test1".to_string(),
                    status: SubmitStatus::Ok,
                },
                TestResults {
                    name: "testy_jawne/test2".to_string(),
                    status: SubmitStatus::Ok,
                },
                TestResults {
                    name: "testy_jawne/test3".to_string(),
                    status: SubmitStatus::Ok,
                },
                TestResults {
                    name: "testy_jawne/test4".to_string(),
                    status: SubmitStatus::Ok,
                },
                TestResults {
                    name: "testy_jawne/test5".to_string(),
                    status: SubmitStatus::Ok,
                },
                TestResults {
                    name: "testy_jawne/test6".to_string(),
                    status: SubmitStatus::Ok,
                },
                TestResults {
                    name: "testy_jawne/test8".to_string(),
                    status: SubmitStatus::Ok,
                },
                TestResults {
                    name: "testy/test0".to_string(),
                    status: SubmitStatus::Ok,
                },
                TestResults {
                    name: "testy/test1".to_string(),
                    status: SubmitStatus::Ok,
                },
                TestResults {
                    name: "testy/test10".to_string(),
                    status: SubmitStatus::TimeExceeded,
                },
                TestResults {
                    name: "testy/test11".to_string(),
                    status: SubmitStatus::TimeExceeded,
                },
                TestResults {
                    name: "testy/test2".to_string(),
                    status: SubmitStatus::Ok,
                },
                TestResults {
                    name: "testy/test3".to_string(),
                    status: SubmitStatus::Ok,
                },
                TestResults {
                    name: "testy/test4".to_string(),
                    status: SubmitStatus::Ok,
                },
                TestResults {
                    name: "testy/test5".to_string(),
                    status: SubmitStatus::Ok,
                },
                TestResults {
                    name: "testy/test6".to_string(),
                    status: SubmitStatus::Ok,
                },
                TestResults {
                    name: "testy/test7".to_string(),
                    status: SubmitStatus::Ok,
                },
                TestResults {
                    name: "testy/test8".to_string(),
                    status: SubmitStatus::Ok,
                },
                TestResults {
                    name: "testy/test9".to_string(),
                    status: SubmitStatus::Ok,
                },
            ]),
        };

        actual.print_with_tests();
        assert_eq!(actual, expected);
    }
}
