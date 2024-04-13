import sys
import random

import chess
import chess.polyglot

if __name__ == "__main__":
	board = chess.Board()

	# This file path is because this python script gets called from a separate directory
	with chess.polyglot.open_reader("src/texel/Perfect2021.bin") as reader:
		number_of_book_moves = random.randint(6, 30)
		for i in range(number_of_book_moves):
			try:
				move = reader.choice(board).move
				board.push(move)
			except:
				break


	print(board.fen())