branch="alex/benching"
echo $(git rev-list --since="1 day ago" --pretty='format:%ad__%h' --date=short $branch | awk 'NR%2==0')
for date_and_commit in $(git rev-list --since="1 week ago" --pretty='format:%ad__%h' --date=short $branch | awk 'NR%2==0')
do
  echo "benching commit" $date_and_commit
  cargo criterion --message-format=json --history-id $date_and_commit > $date_and_commit.json
done

