// DEBUG CODE
TopBanana_set_server_url("http://localhost:8000/");
var game = TopBanana_new_game("8a44c06d-89f3-482a-af62-2aa7f1c5eb6f", "ZDaG4kLlpK14kgQPATEsh45DzbBkKKfLJY8RyaxrJfdsnD2F8AcAWY5B1LtWadlQgsmaF1yDjrDCLZos_SXd4A");
TopBanana_get_scores(game, "9dde10a9-d5ed-4c38-9f3e-7ce847c6e0f1", 3, function(scores) {
  show_debug_message(scores);
});

TopBanana_submit_score(game, "9dde10a9-d5ed-4c38-9f3e-7ce847c6e0f1", "Player 1", 8, "meta", function() {
  show_debug_message("Done.");
});
