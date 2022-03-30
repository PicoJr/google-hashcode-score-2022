#!/bin/bash
curl --silent --request POST --data-binary @out/a_an_example.in.txt.out http://localhost:3000/score/0 | jq .score
curl --silent --request POST --data-binary @out/b_better_start_small.in.txt.out http://localhost:3000/score/1 | jq .score
curl --silent --request POST --data-binary @out/c_collaboration.in.txt.out http://localhost:3000/score/2 | jq .score
curl --silent --request POST --data-binary @out/d_dense_schedule.in.txt.out http://localhost:3000/score/3 | jq .score
curl --silent --request POST --data-binary @out/e_exceptional_skills.in.txt.out http://localhost:3000/score/4 | jq .score
curl --silent --request POST --data-binary @out/f_find_great_mentors.in.txt.out http://localhost:3000/score/5 | jq .score
