#include "wildcard.hh"
#include <string.h>

// Copied from https://www.geeksforgeeks.org/wildcard-pattern-matching/
// Modified to use std::string

bool strmatch(std::string str, std::string pattern) {
  // empty pattern can only match with empty string
  if (pattern.length() == 0)
    return (str.length() == 0);

  // lookup table for storing results of subproblems
  bool lookup[str.length() + 1][pattern.length() + 1];

  // initialize lookup table to false
  memset(lookup, false, sizeof(lookup));

  // empty pattern can match with empty string
  lookup[0][0] = true;

  // Only '*' can match with empty string
  for (std::string::size_type j = 1; j <= pattern.length(); j++)
    if (pattern[j - 1] == '*')
      lookup[0][j] = lookup[0][j - 1];

  // fill the table in bottom-up fashion
  for (std::string::size_type i = 1; i <= str.length(); i++) {
    for (std::string::size_type j = 1; j <= pattern.length(); j++) {
      // Two cases if we see a '*'
      // a) We ignore -F¡*¢ character and move-A
      //    to next  character in the pattern,
      //     i.e., -F¡*¢ indicates an empty sequence.-A
      // b) '*' character matches with ith
      //     character in input
      if (pattern[j - 1] == '*')
	lookup[i][j] = lookup[i][j - 1] || lookup[i - 1][j];

      // Current characters are considered as
      // matching in two cases
      // (a) current character of pattern is '?'
      // (b) characters actually match
      else if (pattern[j - 1] == '?' || str[i - 1] == pattern[j - 1])
	lookup[i][j] = lookup[i - 1][j - 1];

      // If characters don't match
      else
	lookup[i][j] = false;
    }
  }

  return lookup[str.length()][pattern.length()];
}
