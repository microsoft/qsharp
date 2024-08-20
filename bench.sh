branch=main
echo $(git rev-list --since="1 week ago" $branch)
for commit in $(git rev-list --since="1 week ago" $branch)
do
  echo "benching commit" . $commit
  cargo criterion --history-id $commit
done
