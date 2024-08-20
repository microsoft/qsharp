branch="alex/benching"
commit_one_week_ago=$(git rev-list -n 1 --before="1 week ago" $branch)
latest_commit=$(git rev-list -n 1 HEAD)

echo "benching commit $commit_one_week_ago"
cargo criterion --message-format=json --history-id $commit_one_week_ago > week_ago.json

echo "benching commit $latest_commit"
cargo criterion --message-format=json --history-id $latest_commit > latest.json
