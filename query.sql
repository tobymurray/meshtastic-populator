SELECT
    ST_Y(location) AS latitude,
    ST_X(location) AS longitude
FROM
    positions
WHERE user_id = '2885175728'
ORDER BY timestamp desc;

SELECT
    CONCAT('[', ST_Y(location), ', ', ST_X(location), ']') AS position
FROM
    positions
WHERE
    (user_id, timestamp) IN (
        SELECT DISTINCT ON (user_id)
            user_id,
            MAX(timestamp) AS timestamp
        FROM
            positions
        GROUP BY
            user_id
    )
ORDER BY user_id;

SELECT DISTINCT ON (user_id)
    user_id, 
    ST_Y(location) AS latitude,
    ST_X(location) AS longitude
FROM
    positions
ORDER BY user_id, timestamp DESC;


SELECT 
    DISTINCT ON (user_id)
    CONCAT('[',
        user_id,
        ', "', 
        TO_CHAR(timestamp, 'YYYY-MM-DD"T"HH24:MI:SS'),
        '", ', 
        ROUND(ST_Y(location)::numeric, 5), 
        ', ', 
        ROUND(ST_X(location)::numeric, 5), 
        '],') 
        AS position
FROM
    positions
where ST_Y(location) != 0 OR ST_X(location) != 0
ORDER BY user_id, timestamp DESC;