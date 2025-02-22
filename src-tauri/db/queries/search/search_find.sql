SELECT *, bm25(search) as score
FROM search
WHERE search MATCH ?1 || '*'
ORDER BY score
LIMIT 5;